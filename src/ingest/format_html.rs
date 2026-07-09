//! Self-contained visual HTML report for `trs ingest --html`.
//!
//! Reuses the ingest pipeline's collected data (files + LOC + symbols + the
//! resolved dependency graph) and renders a single self-contained page: KPIs,
//! LOC-by-module bars, an interactive force-directed module graph, and an
//! oversized-file flag list. No external assets — CSS/JS are inlined so the
//! file opens anywhere and survives a strict CSP.

use std::collections::HashMap;
use std::path::Path;

use super::deps::build_dep_graph;
use super::mod_html::{CSS, GRAPH_JS};
use super::DigestFile;

/// Group a file into a display "module" = its directory (so files in the same
/// folder share a node), with a single leading source-root wrapper
/// (`src`/`lib`/`app`) stripped for tidier labels. A top-level file maps to its
/// stem (`src/main.rs` → `main`).
///
/// Grouping by directory (not by the first component) is what keeps the graph
/// meaningful for monorepo / multi-root layouts: collapsing everything under
/// `docu_frontend/` into one node would turn every real edge into a
/// self-loop and empty the graph.
fn module_of(rel: &str) -> String {
    let mut parts: Vec<&str> = rel.split('/').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return "(root)".to_string();
    }
    if parts.len() > 1 && matches!(parts[0], "src" | "lib" | "app") {
        parts.remove(0);
    }
    if parts.len() == 1 {
        // top-level file → its stem
        Path::new(parts[0])
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(parts[0])
            .to_string()
    } else {
        // nested → the containing directory path (drop the filename)
        parts[..parts.len() - 1].join("/")
    }
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn human(n: usize) -> String {
    if n >= 1000 {
        let k = n as f64 / 1000.0;
        let s = format!("{:.1}", k);
        format!("{}k", s.trim_end_matches(".0"))
    } else {
        n.to_string()
    }
}

/// Render the full report page.
pub(super) fn format_html(
    files: &[DigestFile],
    project_name: &str,
    root: &str,
    max_loc: usize,
) -> String {
    let real: Vec<&DigestFile> = files.iter().filter(|f| !f.rel_path.is_empty()).collect();
    let total_loc: usize = real.iter().map(|f| f.loc).sum();
    let file_count = real.len();
    let symbol_count: usize = real.iter().map(|f| f.symbols.len()).sum();

    // --- per-module LOC + file count ---
    let mut mod_loc: HashMap<String, usize> = HashMap::new();
    let mut mod_files: HashMap<String, usize> = HashMap::new();
    for f in &real {
        let m = module_of(&f.rel_path);
        *mod_loc.entry(m.clone()).or_default() += f.loc;
        *mod_files.entry(m).or_default() += 1;
    }

    // --- LOC-by-module bars (top 11) ---
    let mut bars: Vec<(&String, usize)> = mod_loc.iter().map(|(m, l)| (m, *l)).collect();
    bars.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    bars.truncate(11);
    let bar_max = bars.first().map(|(_, l)| *l).unwrap_or(1).max(1);
    let bars_json = bars
        .iter()
        .map(|(name, loc)| {
            format!(
                r#"{{"name":{},"loc":{},"files":{}}}"#,
                json_str(name),
                loc,
                mod_files.get(*name).copied().unwrap_or(0)
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    // --- file distribution by extension (top 9) ---
    let mut ext_count: HashMap<String, usize> = HashMap::new();
    for f in &real {
        let e = Path::new(&f.rel_path)
            .extension()
            .and_then(|x| x.to_str())
            .unwrap_or("—")
            .to_lowercase();
        *ext_count.entry(e).or_default() += 1;
    }
    let mut exts: Vec<(String, usize)> = ext_count.into_iter().collect();
    exts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let ext_html = exts
        .iter()
        .take(9)
        .map(|(e, c)| format!(r#"<span class="chip">.{} <b>{}</b></span>"#, esc(e), c))
        .collect::<Vec<_>>()
        .join("");

    // --- dependency graph, aggregated to module level ---
    let graph = build_dep_graph(files);
    let mut medges: HashMap<(String, String), usize> = HashMap::new();
    for (src, dsts) in &graph.edges {
        let ms = module_of(src);
        for d in dsts {
            let md = module_of(d);
            if ms != md {
                *medges.entry((ms.clone(), md.clone())).or_default() += 1;
            }
        }
    }
    let mut deg: HashMap<String, usize> = HashMap::new();
    for (s, t) in medges.keys() {
        *deg.entry(s.clone()).or_default() += 1;
        *deg.entry(t.clone()).or_default() += 1;
    }
    let mut ranked: Vec<String> = deg.keys().cloned().collect();
    ranked.sort_by(|a, b| {
        deg[b]
            .cmp(&deg[a])
            .then(mod_loc.get(b).cmp(&mod_loc.get(a)))
            .then(a.cmp(b))
    });
    let keep: std::collections::HashSet<String> = ranked.iter().take(22).cloned().collect();
    let nodes_json = keep
        .iter()
        .map(|m| {
            format!(
                r#"{{"id":{},"loc":{},"deg":{},"files":{}}}"#,
                json_str(m),
                mod_loc.get(m).copied().unwrap_or(0),
                deg.get(m).copied().unwrap_or(0),
                mod_files.get(m).copied().unwrap_or(0)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let kept_edges: Vec<&(String, String)> = medges
        .keys()
        .filter(|(s, t)| keep.contains(s) && keep.contains(t))
        .collect();
    let edges_json = kept_edges
        .iter()
        .map(|(s, t)| format!("[{},{}]", json_str(s), json_str(t)))
        .collect::<Vec<_>>()
        .join(",");
    let edge_count = kept_edges.len();

    // --- oversized files ---
    let mut over: Vec<&&DigestFile> = real.iter().filter(|f| f.loc > max_loc).collect();
    over.sort_by_key(|f| std::cmp::Reverse(f.loc));
    let over_count = over.len();
    let over_rows = if over.is_empty() {
        format!(
            r#"<div class="row"><span class="p">No files over {} LOC — tidy.</span></div>"#,
            max_loc
        )
    } else {
        over.iter()
            .take(14)
            .map(|f| {
                format!(
                    r#"<div class="row"><span class="p">{}</span><span class="loc">{}</span></div>"#,
                    esc(&f.rel_path),
                    f.loc
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    // Note: a proper "unreferenced / dead code" section needs real
    // reachability (mod-wiring, method calls, trait impls, macros) — import
    // edges alone flag ~half a Rust crate as false orphans. Deferred to a
    // dedicated `--flag-unused` pass rather than shipped noisy here.

    // --- assemble ---
    let kpis = format!(
        r#"<div class="kpi"><span class="n tnum">{}</span><span class="l"><b>lines</b> · {} files</span></div>
    <div class="kpi"><span class="n tnum">{}</span><span class="l"><b>symbols</b> indexed</span></div>
    <div class="kpi"><span class="n tnum">{}</span><span class="l">modules <b>graphed</b></span></div>
    <div class="kpi"><span class="n tnum">{}</span><span class="l">files <b>over {} LOC</b></span></div>"#,
        human(total_loc),
        file_count,
        human(symbol_count),
        keep.len(),
        over_count,
        max_loc
    );

    let over_class = if over_count > 0 { "warn" } else { "" };
    let over_pill = if over_count > 0 {
        format!(r#"<span class="pill warn">{} files</span>"#, over_count)
    } else {
        r#"<span class="pill good">clean</span>"#.to_string()
    };

    let body = format!(
        r##"<div class="wrap">
  <header>
    <div class="brand">
      <span class="eyebrow">Codebase report</span>
      <h1>{name}</h1>
      <span class="path mono">{root}</span>
      <p class="sub">Structure, size and internal dependencies at a glance — generated by <code style="font-family:var(--mono)">trs ingest --html</code>.</p>
    </div>
    <span class="ver">{files} files · {loc} LOC</span>
  </header>

  <div class="kpis">{kpis}</div>

  <div class="distrow">
    <span class="dlabel">file mix</span>
    <div class="chips">{exts}</div>
  </div>

  <section>
    <div class="h"><h2>Where the code lives</h2><span class="tag">LOC &amp; files by module</span></div>
    <p class="lead">Each module is a folder (or a top-level file). Bar = lines of code; the count shows how many files it holds.</p>
    <div class="bars" id="bars"></div>
  </section>

  <section>
    <div class="h"><h2>How it connects</h2><span class="tag">module graph · {nnodes} nodes · {nedges} edges</span></div>
    <p class="lead">Real internal <code style="font-family:var(--mono)">import / use</code> dependencies between modules. <b style="color:var(--ink)">Circle size = lines of code</b>; <b style="color:var(--ink)">color = connectivity</b> (teal = hub). Hover to preview, <b style="color:var(--ink)">click a node to pin</b> its links; drag to rearrange.</p>
    <div class="graph-wrap">
      <canvas id="graph"></canvas>
      <div class="glegend">
        <div class="k" style="color:var(--faint);font-size:10px;letter-spacing:.08em">COLOR = ROLE</div>
        <div class="k"><span class="sw" style="background:var(--accent)"></span>hub (highly connected)</div>
        <div class="k"><span class="sw" style="background:var(--muted)"></span>module</div>
        <div class="k" style="color:var(--faint);font-size:10px;letter-spacing:.08em;margin-top:3px">SIZE = LOC</div>
      </div>
      <div class="ghint" id="ghint">hover a node · click to pin</div>
    </div>
  </section>

  <section>
    <div class="h"><h2>Oversized files</h2><span class="tag">over {maxloc} LOC</span></div>
    <p class="lead">Long files are harder to hold in one head. Tune the threshold with <code style="font-family:var(--mono)">--max-loc N</code>.</p>
    <div class="card {overclass}">
      <h3><span class="dot warn"></span>Files to consider splitting {overpill}</h3>
      <div class="rows">
{overrows}
      </div>
    </div>
  </section>

  <footer>
    <span>Snapshot of <b style="color:var(--ink)">{name}</b> · {files} files ingested</span>
    <span>generated by <code>trs ingest --html</code></span>
  </footer>
</div>"##,
        name = esc(project_name),
        root = esc(root),
        files = file_count,
        loc = human(total_loc),
        kpis = kpis,
        exts = ext_html,
        nnodes = keep.len(),
        nedges = edge_count,
        maxloc = max_loc,
        overclass = over_class,
        overpill = over_pill,
        overrows = over_rows,
    );

    let script = format!(
        "const BAR_MAX={bar_max};const BARS=[{bars}];const GN=[{nodes}];const GE=[{edges}];\n{js}",
        bar_max = bar_max,
        bars = bars_json,
        nodes = nodes_json,
        edges = edges_json,
        js = GRAPH_JS,
    );

    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
<title>{} — codebase report</title><style>{}</style></head><body>{}<script>{}</script></body></html>",
        esc(project_name),
        CSS,
        body,
        script
    )
}

/// Minimal JSON string encoder for the embedded data literals.
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}
