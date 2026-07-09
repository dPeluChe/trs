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

/// The project's own one-line purpose: manifest `description` first (the most
/// deliberate statement), else the README's first prose paragraph. Gives the
/// agent *intent*, not just structure.
fn project_about(files: &[&DigestFile]) -> Option<String> {
    let named = |f: &&DigestFile, want: &str| {
        Path::new(&f.rel_path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.eq_ignore_ascii_case(want))
            .unwrap_or(false)
    };
    for f in files {
        if named(f, "Cargo.toml") || named(f, "pyproject.toml") {
            if let Some(d) = kv_value(&f.content, "description") {
                return Some(d);
            }
        }
        if named(f, "package.json") {
            if let Some(d) = json_value(&f.content, "description") {
                return Some(d);
            }
        }
    }
    for f in files {
        if named(f, "README.md") {
            if let Some(p) = readme_first_para(&f.content) {
                return Some(p);
            }
        }
    }
    None
}

/// `key = "value"` / `key = 'value'` from a TOML-ish blob (first match).
fn kv_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                let v = rest.trim().trim_matches(|c| c == '"' || c == '\'').trim();
                if !v.is_empty() {
                    return Some(truncate(v, 240));
                }
            }
        }
    }
    None
}

/// `"key": "value"` from a JSON blob (first match).
fn json_value(content: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let idx = content.find(&needle)?;
    let after = &content[idx + needle.len()..];
    let colon = after.find(':')?;
    let after = after[colon + 1..].trim_start();
    let after = after.strip_prefix('"')?;
    let end = after.find('"')?;
    let v = after[..end].trim();
    if v.is_empty() {
        None
    } else {
        Some(truncate(v, 240))
    }
}

/// First real prose paragraph of a README (skip headings, badges, images, HTML).
fn readme_first_para(content: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if t.is_empty()
            || t.starts_with('#')
            || t.starts_with('!')
            || t.starts_with('[')
            || t.starts_with('<')
            || t.starts_with('>')
            || t.starts_with("---")
            || t.starts_with("```")
            || t.contains('|') // status/table rows ("Versión: x | Estado: y")
            || t.contains("://") && t.split_whitespace().count() <= 2
        // a bare URL line
        {
            continue;
        }
        // Strip inline markdown emphasis/links lightly.
        let clean = t.replace(['*', '`'], "");
        if clean.len() >= 20 {
            return Some(truncate(clean.trim(), 240));
        }
    }
    None
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push('…');
        out
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

fn human_bytes(b: u64) -> String {
    const U: &[&str] = &["B", "KB", "MB", "GB"];
    let mut v = b as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} B", b)
    } else {
        format!("{:.1} {}", v, U[i])
    }
}

/// Image / media / binary assets are skipped by the digest (they aren't
/// code), but they carry real weight in a repo. A second, gitignore-aware
/// walk tallies them by category + size and surfaces the heaviest files.
/// Returns `(section_html, total_count, total_bytes)`.
fn scan_assets(root: &Path) -> (String, usize, u64) {
    const CATS: &[(&str, &[&str])] = &[
        (
            "images",
            &[
                "png", "jpg", "jpeg", "gif", "webp", "svg", "ico", "bmp", "avif", "heic",
            ],
        ),
        (
            "media",
            &[
                "mp4", "mov", "mp3", "wav", "avi", "mkv", "webm", "m4a", "flac",
            ],
        ),
        ("fonts", &["woff", "woff2", "ttf", "eot", "otf"]),
        ("archives", &["zip", "tar", "gz", "bz2", "xz", "7z", "rar"]),
        (
            "pdf/office",
            &["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx"],
        ),
        (
            "data/bin",
            &["sqlite", "sqlite3", "db", "parquet", "bin", "wasm"],
        ),
    ];
    let mut count: HashMap<&str, usize> = HashMap::new();
    let mut bytes: HashMap<&str, u64> = HashMap::new();
    let mut heavy: Vec<(String, u64)> = Vec::new();
    let mut builder = ignore::WalkBuilder::new(root);
    builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);
    for entry in builder.build().flatten() {
        let path = entry.path();
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(true) {
            continue;
        }
        if path.components().any(|c| {
            super::SKIP_DIRS
                .iter()
                .any(|d| c.as_os_str().to_str() == Some(*d))
        }) {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        for (cat, exts) in CATS {
            if exts.contains(&ext.as_str()) {
                let sz = entry.metadata().ok().map(|m| m.len()).unwrap_or(0);
                *count.entry(cat).or_default() += 1;
                *bytes.entry(cat).or_default() += sz;
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .to_string();
                heavy.push((rel, sz));
                break;
            }
        }
    }
    let total_count: usize = count.values().sum();
    let total_bytes: u64 = bytes.values().sum();
    let mut cats: Vec<&str> = count.keys().copied().collect();
    cats.sort_by_key(|c| std::cmp::Reverse(bytes.get(c).copied().unwrap_or(0)));
    let chips = cats
        .iter()
        .map(|c| {
            format!(
                r#"<span class="chip">{} <b>{}</b> · {}</span>"#,
                c,
                count[c],
                human_bytes(bytes[c])
            )
        })
        .collect::<Vec<_>>()
        .join("");
    heavy.sort_by_key(|(_, b)| std::cmp::Reverse(*b));
    let rows = if heavy.is_empty() {
        r#"<div class="row"><span class="p">No image / media / binary assets.</span></div>"#
            .to_string()
    } else {
        heavy
            .iter()
            .take(12)
            .map(|(p, b)| {
                format!(
                    r#"<div class="row"><span class="p">{}</span><span class="loc">{}</span></div>"#,
                    esc(p),
                    human_bytes(*b)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let html = format!(
        r#"<div class="chips" style="margin-bottom:16px">{}</div><div class="rows">{}</div>"#,
        chips, rows
    );
    (html, total_count, total_bytes)
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
    let about_line = match project_about(&real) {
        Some(a) => esc(&a),
        None => "Structure, size and internal dependencies at a glance.".to_string(),
    };

    // --- per-module aggregation (LOC, file count, file list) ---
    let mut mod_loc: HashMap<String, usize> = HashMap::new();
    let mut mod_files: HashMap<String, usize> = HashMap::new();
    let mut mod_list: HashMap<String, Vec<(&str, usize)>> = HashMap::new();
    for f in &real {
        let m = module_of(&f.rel_path);
        *mod_loc.entry(m.clone()).or_default() += f.loc;
        *mod_files.entry(m.clone()).or_default() += 1;
        mod_list.entry(m).or_default().push((&f.rel_path, f.loc));
    }

    // --- LOC-by-module bars (top 11), each carrying its own files ---
    let mut bars: Vec<(&String, usize)> = mod_loc.iter().map(|(m, l)| (m, *l)).collect();
    bars.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    bars.truncate(11);
    let bar_max = bars.first().map(|(_, l)| *l).unwrap_or(1).max(1);
    let bars_json = bars
        .iter()
        .map(|(name, loc)| {
            let mut list = mod_list.get(*name).cloned().unwrap_or_default();
            list.sort_by_key(|(_, l)| std::cmp::Reverse(*l));
            let items = list
                .iter()
                .take(16)
                .map(|(p, l)| format!(r#"{{"p":{},"loc":{}}}"#, json_str(p), l))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                r#"{{"name":{},"loc":{},"fc":{},"list":[{}]}}"#,
                json_str(name),
                loc,
                mod_files.get(*name).copied().unwrap_or(0),
                items
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
    // Fan-in (how many distinct modules import me) and fan-out (how many I
    // import). Edges are already deduped to unique (src, dst) pairs.
    let mut in_deg: HashMap<String, usize> = HashMap::new();
    let mut out_deg: HashMap<String, usize> = HashMap::new();
    for (s, t) in medges.keys() {
        *out_deg.entry(s.clone()).or_default() += 1;
        *in_deg.entry(t.clone()).or_default() += 1;
    }
    let mut deg: HashMap<String, usize> = HashMap::new();
    for m in in_deg.keys().chain(out_deg.keys()) {
        deg.entry(m.clone()).or_insert_with(|| {
            in_deg.get(m).copied().unwrap_or(0) + out_deg.get(m).copied().unwrap_or(0)
        });
    }
    // Role by pure fan-in/fan-out topology (classify_layer, no AST):
    //   entry  — nothing imports it, it imports others (a root / CLI / main)
    //   leaf   — used by others, imports nothing (a utility / type module)
    //   core   — high fan-in: many modules route through it
    //   internal — everything else (mid of the graph)
    let core_floor = in_deg.values().copied().max().unwrap_or(0).max(6) / 2;
    let role_of = |m: &str| -> &'static str {
        let i = in_deg.get(m).copied().unwrap_or(0);
        let o = out_deg.get(m).copied().unwrap_or(0);
        if i == 0 && o > 0 {
            "entry"
        } else if o == 0 && i > 0 {
            "leaf"
        } else if i >= core_floor.max(3) {
            "core"
        } else {
            "internal"
        }
    };
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
                r#"{{"id":{},"loc":{},"deg":{},"files":{},"role":{},"in":{},"out":{}}}"#,
                json_str(m),
                mod_loc.get(m).copied().unwrap_or(0),
                deg.get(m).copied().unwrap_or(0),
                mod_files.get(m).copied().unwrap_or(0),
                json_str(role_of(m)),
                in_deg.get(m).copied().unwrap_or(0),
                out_deg.get(m).copied().unwrap_or(0),
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

    // roles breakdown among graphed modules (classify_layer summary)
    let mut role_groups: HashMap<&str, Vec<&str>> = HashMap::new();
    for m in &keep {
        role_groups.entry(role_of(m)).or_default().push(m.as_str());
    }
    let role_desc: &[(&str, &str)] = &[
        ("entry", "roots — nothing imports them (main, CLI)"),
        ("core", "high fan-in — everything routes through"),
        ("leaf", "used by many, import nothing (utils, types)"),
        ("internal", "mid-graph plumbing"),
    ];
    let roles_html = role_desc
        .iter()
        .filter_map(|(r, desc)| {
            role_groups.get(r).map(|mods| {
                let mut ms = mods.to_vec();
                ms.sort_unstable();
                let sample = ms.iter().take(5).copied().collect::<Vec<_>>().join(", ");
                let more = if ms.len() > 5 {
                    format!(" +{}", ms.len() - 5)
                } else {
                    String::new()
                };
                format!(
                    r#"<div class="rolerow"><span class="rolebadge {r}">{r}</span><span class="roled">{d}</span><span class="rolem mono">{s}{m}</span></div>"#,
                    r = r,
                    d = desc,
                    s = esc(&sample),
                    m = more
                )
            })
        })
        .collect::<Vec<_>>()
        .join("");

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

    // --- assets / binaries (images, media, fonts…) the digest skips ---
    let (assets_html, asset_count, asset_bytes) = scan_assets(Path::new(root));

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
      <p class="sub">{about}</p>
      <p class="gen">by <code style="font-family:var(--mono)">trs ingest --html</code></p>
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
    <p class="lead">Real internal <code style="font-family:var(--mono)">import / use</code> dependencies between modules. <b style="color:var(--ink)">Circle size = lines of code</b>; <b style="color:var(--ink)">color = role</b>, derived from fan-in / fan-out (below). Hover to preview, <b style="color:var(--ink)">click a node to pin</b> its links; drag to rearrange.</p>
    <div class="graph-wrap">
      <canvas id="graph"></canvas>
      <div class="glegend">
        <div class="k" style="color:var(--faint);font-size:10px;letter-spacing:.08em">COLOR = ROLE</div>
        <div class="k"><span class="sw" style="background:var(--accent2)"></span>entry</div>
        <div class="k"><span class="sw" style="background:var(--accent)"></span>core</div>
        <div class="k"><span class="sw" style="background:var(--muted)"></span>leaf</div>
        <div class="k"><span class="sw" style="background:var(--faint)"></span>internal</div>
      </div>
      <div class="ghint" id="ghint">hover a node · click to pin</div>
    </div>
    <div class="roles">{roles}</div>
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

  <section>
    <div class="h"><h2>Assets &amp; binaries</h2><span class="tag">{acount} files · {abytes}</span></div>
    <p class="lead">Images, media, fonts and other binaries — skipped by the code digest but real weight in the repo. Heaviest files listed.</p>
    <div class="card">
      <h3><span class="dot warn"></span>What's taking space</h3>
{assets}
    </div>
  </section>

  <footer>
    <span>Snapshot of <b style="color:var(--ink)">{name}</b> · {files} files ingested</span>
    <span>generated by <code>trs ingest --html</code></span>
  </footer>
</div>"##,
        name = esc(project_name),
        root = esc(root),
        about = about_line,
        files = file_count,
        loc = human(total_loc),
        kpis = kpis,
        exts = ext_html,
        nnodes = keep.len(),
        nedges = edge_count,
        roles = roles_html,
        maxloc = max_loc,
        overclass = over_class,
        overpill = over_pill,
        overrows = over_rows,
        acount = asset_count,
        abytes = human_bytes(asset_bytes),
        assets = assets_html,
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
