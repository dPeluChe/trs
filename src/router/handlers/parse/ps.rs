//! `ps aux` / `ps -ef` output compression.
//!
//! `ps aux` on a developer laptop commonly outputs 200-300KB — one
//! line per process, and the `COMMAND` column carries the full
//! rustc/clang invocation that can be megabytes long by itself.
//! Agents rarely need any of that; they're usually scanning for a
//! specific process or grepping CPU/memory hogs.
//!
//! This handler parses the header to find column positions, then for
//! each row keeps USER/PID/%CPU/%MEM/STAT + a one-token command
//! (basename of argv[0]), sorts descending by CPU, and truncates long
//! rosters to the top 30. Full roster + raw command lines stay
//! available via `--raw` / `--json` if the agent actually needs them.

use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

#[derive(Debug, Clone)]
struct Proc {
    user: String,
    pid: String,
    cpu: String,
    mem: String,
    stat: String,
    cmd: String,
}

impl ParseHandler {
    pub(crate) fn handle_ps(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let input_bytes = input.len();

        let procs = match parse_ps(&input) {
            Some(p) => p,
            None => {
                // Unrecognized header → passthrough. Keeps the handler
                // honest on `ps -o pid,cmd` / non-BSD output / etc.
                print!("{}", input);
                if ctx.stats {
                    CommandStats::new()
                        .with_reducer("ps-passthrough")
                        .with_input_bytes(input_bytes)
                        .with_output_bytes(input_bytes)
                        .print();
                }
                return Ok(());
            }
        };

        let output = match ctx.format {
            OutputFormat::Json => render_json(&procs),
            OutputFormat::Raw => input.clone(),
            _ => render_compact(&procs),
        };

        print!("{}", output);

        if ctx.stats {
            CommandStats::new()
                .with_reducer("ps")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}

/// Parse `ps aux` / `ps -ef` into a Proc list. Returns None when the
/// header doesn't look like `ps` output at all, which signals the
/// caller to pass through unchanged.
fn parse_ps(input: &str) -> Option<Vec<Proc>> {
    let mut lines = input.lines();
    let header = lines.next()?;
    let cols = header.split_whitespace().collect::<Vec<_>>();

    // Require a recognizable header — `ps aux` and `ps -ef` layouts
    // both have USER/UID + PID + %CPU or CMD near the start.
    let has_pid = cols.iter().any(|c| c.eq_ignore_ascii_case("PID"));
    let has_user_or_uid = cols
        .iter()
        .any(|c| c.eq_ignore_ascii_case("USER") || c.eq_ignore_ascii_case("UID"));
    if !has_pid || !has_user_or_uid {
        return None;
    }

    // `ps` right-aligns numeric columns under their header labels, so
    // byte offsets from the header don't line up with data rows.
    // Instead, count whitespace-separated fields: COMMAND is always
    // the last column, so splitting a row into (COMMAND_idx) leading
    // fields + everything-else-as-the-command gives us a clean parse.
    let user_idx = find_col(&cols, &["USER", "UID"]);
    let pid_idx = find_col(&cols, &["PID"]);
    let cpu_idx = find_col(&cols, &["%CPU", "C"]);
    let mem_idx = find_col(&cols, &["%MEM"]);
    let stat_idx = find_col(&cols, &["STAT", "S"]);
    let cmd_idx = find_col(&cols, &["COMMAND", "CMD"])?;

    let mut procs = Vec::new();
    for raw in lines {
        if raw.trim().is_empty() {
            continue;
        }
        let (head_fields, tail) = split_at_field(raw, cmd_idx);
        let get = |idx: Option<usize>| -> String {
            idx.and_then(|i| head_fields.get(i))
                .map(|s| s.to_string())
                .unwrap_or_else(|| "?".into())
        };
        procs.push(Proc {
            user: get(user_idx),
            pid: get(pid_idx),
            cpu: get(cpu_idx),
            mem: get(mem_idx),
            stat: get(stat_idx),
            cmd: shorten_command(tail.trim()),
        });
    }
    Some(procs)
}

/// Split `row` into `(first N whitespace-separated fields, rest)`.
/// The "rest" keeps its internal whitespace intact — crucial when the
/// COMMAND column contains spaces (as it usually does).
fn split_at_field(row: &str, n: usize) -> (Vec<&str>, &str) {
    let bytes = row.as_bytes();
    let mut fields: Vec<&str> = Vec::with_capacity(n);
    let mut i = 0;
    while fields.len() < n && i < bytes.len() {
        while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let start = i;
        while i < bytes.len() && bytes[i] != b' ' && bytes[i] != b'\t' {
            i += 1;
        }
        fields.push(&row[start..i]);
    }
    while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t') {
        i += 1;
    }
    (fields, &row[i..])
}

/// Find the zero-based column index for any of `names` in the header.
fn find_col(cols: &[&str], names: &[&str]) -> Option<usize> {
    cols.iter()
        .position(|c| names.iter().any(|n| c.eq_ignore_ascii_case(n)))
}

/// Reduce a full COMMAND string to its identifying tail. For a path
/// like `/Users/.../bin/rustc --crate-name ... -L ...`, we keep
/// `rustc` plus up to one short-looking positional argument. The
/// heuristic isn't perfect but strips the 99% case: multi-kilobyte
/// rustc invocations.
fn shorten_command(cmd: &str) -> String {
    // Tokenize once; an empty cmd stays as "".
    let first = cmd.split_whitespace().next().unwrap_or("");
    if first.is_empty() {
        return String::new();
    }
    let exe = first.rsplit('/').next().unwrap_or(first);

    // Append the second token only if it looks like a readable
    // positional (not a flag, not a megabyte-long blob). Otherwise the
    // bare executable name is enough — the PID already identifies the
    // specific process.
    let second = cmd.split_whitespace().nth(1).unwrap_or("");
    if !second.is_empty() && !second.starts_with('-') && second.len() <= 40 {
        format!("{} {}", exe, second)
    } else {
        exe.to_string()
    }
}

/// Render the compact view: top 30 by CPU, with a summary footer.
fn render_compact(procs: &[Proc]) -> String {
    let mut sorted: Vec<&Proc> = procs.iter().collect();
    sorted.sort_by(|a, b| {
        b.cpu
            .parse::<f32>()
            .unwrap_or(0.0)
            .partial_cmp(&a.cpu.parse::<f32>().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total = procs.len();
    let idle = procs
        .iter()
        .filter(|p| matches!(p.stat.as_str(), "S" | "I" | "IW" | "Is"))
        .count();

    let top = &sorted[..sorted.len().min(30)];
    let mut out = String::new();
    out.push_str(&format!(
        "ps: {} processes ({} sleeping/idle). Top by CPU:\n",
        total, idle
    ));
    for p in top {
        out.push_str(&format!(
            "  {:>8} {:>6} {:>5}% cpu {:>5}% mem  {:<4}  {}\n",
            p.user, p.pid, p.cpu, p.mem, p.stat, p.cmd
        ));
    }
    if sorted.len() > 30 {
        out.push_str(&format!(
            "  ... {} more (use --raw or --json for the full list)\n",
            sorted.len() - 30
        ));
    }
    out
}

fn render_json(procs: &[Proc]) -> String {
    let entries: Vec<serde_json::Value> = procs
        .iter()
        .map(|p| {
            serde_json::json!({
                "user": p.user,
                "pid": p.pid,
                "cpu": p.cpu,
                "mem": p.mem,
                "stat": p.stat,
                "cmd": p.cmd,
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "count": procs.len(),
        "processes": entries,
    }))
    .unwrap_or_default()
        + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
USER               PID  %CPU %MEM      VSZ    RSS   TT  STAT STARTED      TIME COMMAND
alice              526  96.1  2.6 414334832 1759456   ??  R     5:32PM   0:12.44 /Users/alice/.rustup/toolchains/stable/bin/rustc --crate-name foo --edition 2021
alice              100   0.0  0.1 400000 10000   ??  S     9:00AM   0:00.01 /usr/sbin/mdnsresponder
alice              200   0.5  1.0 500000 80000   ??  R    10:00AM   0:05.00 /Applications/Arc.app/Contents/MacOS/Arc
";

    #[test]
    fn parses_ps_aux_sample() {
        let procs = parse_ps(SAMPLE).unwrap();
        assert_eq!(procs.len(), 3);
        assert_eq!(procs[0].user, "alice");
        assert_eq!(procs[0].pid, "526");
        assert_eq!(procs[0].cpu, "96.1");
        // Basename kept; flag-leading args are dropped so the command
        // stays terse (--crate-name triggers the flag short-circuit).
        assert_eq!(procs[0].cmd, "rustc");
    }

    #[test]
    fn compact_is_shorter_than_raw() {
        let procs = parse_ps(SAMPLE).unwrap();
        let out = render_compact(&procs);
        assert!(
            out.len() < SAMPLE.len(),
            "compact not shorter: {} vs {}",
            out.len(),
            SAMPLE.len()
        );
        assert!(out.contains("ps: 3 processes"));
        assert!(out.contains("rustc"));
    }

    #[test]
    fn passthrough_on_unknown_header() {
        let custom = "not a ps header\nsome data\n";
        assert!(parse_ps(custom).is_none());
    }

    #[test]
    fn shorten_command_strips_huge_rustc_args() {
        let long = "/Users/a/.rustup/toolchains/stable/bin/rustc --crate-name foo --edition 2021 -C opt-level=3 -L /very/long/path -L /another/path --extern libc=/more/path/libc.rlib";
        let short = shorten_command(long);
        assert!(short.starts_with("rustc"));
        assert!(short.len() < 50);
    }
}
