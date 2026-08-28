//! Compression for three system-inventory commands that share one shape:
//! many rows, a few columns that matter, and heavy repetition.
//!
//! - `du`: one row per path. Agents want the big ones, so rows are sorted
//!   descending and the tail is summarized instead of printed.
//! - `lsof`: one row per file descriptor, so a single process with 40 open
//!   sockets is 40 near-identical rows. Rows are folded per process and only
//!   the NAME column (the address) survives; DEVICE/SIZE/OFF/NODE never
//!   answer the question agents actually ask ("what is on this port?").
//! - `pgrep -fl`: one row per pid, and identical commands repeat verbatim.
//!   Equal command lines collapse into one row carrying every pid.
//!
//! Every handler passes the input straight through when the shape does not
//! match, so an unusual flag combination degrades to raw rather than to a
//! confidently wrong summary.

use super::super::common::{CommandContext, CommandResult};
use super::ParseHandler;

const DU_ROWS: usize = 15;
const LSOF_ROWS: usize = 25;
const PGREP_ROWS: usize = 25;
const CMD_MAX: usize = 90;

/// Bytes for a `du` size cell, human (`8.0K`, `1.2G`) or plain block count.
/// Returns None for a cell that is neither, which drops the row to passthrough.
fn size_bytes(cell: &str) -> Option<f64> {
    let c = cell.trim();
    if c.is_empty() {
        return None;
    }
    let (num, mult) = match c.chars().last()? {
        'B' => (&c[..c.len() - 1], 1.0),
        'K' => (&c[..c.len() - 1], 1024.0),
        'M' => (&c[..c.len() - 1], 1024.0 * 1024.0),
        'G' => (&c[..c.len() - 1], 1024.0 * 1024.0 * 1024.0),
        'T' => (&c[..c.len() - 1], 1024.0f64.powi(4)),
        '0'..='9' => (c, 1024.0), // `du -s` reports 1K blocks
        _ => return None,
    };
    num.trim().parse::<f64>().ok().map(|n| n * mult)
}

/// Shorten a command line: argv[0] to its basename, arguments kept, then a
/// hard cap. A 200-char npx path says nothing the basename does not.
fn short_cmd(cmd: &str) -> String {
    let mut parts = cmd.split_whitespace();
    let head = parts.next().unwrap_or("");
    let base = head.rsplit('/').next().unwrap_or(head);
    let rest: Vec<&str> = parts.collect();
    let joined = if rest.is_empty() {
        base.to_string()
    } else {
        format!("{} {}", base, rest.join(" "))
    };
    if joined.chars().count() > CMD_MAX {
        let cut: String = joined.chars().take(CMD_MAX).collect();
        format!("{}…", cut)
    } else {
        joined
    }
}

/// `du` rows sorted by size descending, tail summarized. None when the shape
/// is not du output, or when there is nothing to sort (0 or 1 row).
fn compress_du(input: &str) -> Option<String> {
    let mut rows: Vec<(f64, String)> = Vec::new();
    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        // du separates with a tab, but some implementations use spaces.
        let (size, path) = line
            .split_once('\t')
            .or_else(|| line.trim_start().split_once(char::is_whitespace))?;
        rows.push((size_bytes(size)?, path.trim().to_string()));
    }
    // One row is already minimal (`du -sh .`); zero means nothing matched.
    if rows.len() < 2 {
        return None;
    }

    let total: f64 = rows.iter().map(|(b, _)| b).sum();
    rows.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut out = String::new();
    for (b, path) in rows.iter().take(DU_ROWS) {
        out.push_str(&format!(
            "{:>7}  {}\n",
            ParseHandler::format_human_size(*b as u64),
            path
        ));
    }
    if rows.len() > DU_ROWS {
        let rest: f64 = rows.iter().skip(DU_ROWS).map(|(b, _)| b).sum();
        out.push_str(&format!(
            "… +{} smaller ({})\n",
            rows.len() - DU_ROWS,
            ParseHandler::format_human_size(rest as u64)
        ));
    }
    out.push_str(&format!(
        "{} entries, {} total\n",
        rows.len(),
        ParseHandler::format_human_size(total as u64)
    ));
    Some(out)
}

/// One row per process instead of one per descriptor, keeping only NAME.
/// None without the standard header (`lsof -F` machine format, or an error).
fn compress_lsof(input: &str) -> Option<String> {
    let mut lines = input.lines();
    let header = lines
        .next()
        .filter(|h| h.starts_with("COMMAND") && h.contains("NAME"))?;
    let name_col = header.find("NAME")?;
    // With `-i`, NODE holds the protocol (TCP/UDP), which is worth keeping.
    // Without it, NODE is an inode number, which is not. Decide per row.
    let node_col = header.find("NODE");

    // (command, pid, user) -> distinct NAME cells, insertion-ordered because
    // the output should read in the order lsof reported. The map is an index
    // into that Vec, not a replacement: a linear `find` per row is quadratic,
    // and `lsof` bare is ~30k rows across ~900 processes.
    let mut groups: Vec<((&str, &str, &str), Vec<String>)> = Vec::new();
    let mut index: std::collections::HashMap<(&str, &str, &str), usize> =
        std::collections::HashMap::new();
    let mut fds = 0usize;
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let mut cols = line.split_whitespace();
        let (Some(cmd), Some(pid), Some(user)) = (cols.next(), cols.next(), cols.next()) else {
            continue;
        };
        // NAME is last and can contain spaces ("TCP *:56165 (LISTEN)"), so
        // slice at the header offset rather than splitting on whitespace.
        let addr = line
            .get(name_col..)
            .map(str::trim)
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| line.split_whitespace().last().unwrap_or(""));
        let proto = node_col
            .and_then(|c| line.get(c..name_col))
            .map(str::trim)
            .filter(|n| n.len() >= 3 && n.chars().all(|c| c.is_ascii_uppercase()));
        let name = match proto {
            Some(p) => format!("{p} {addr}"),
            None => addr.to_string(),
        };
        fds += 1;
        let key = (cmd, pid, user);
        match index.get(&key) {
            Some(&i) => {
                let names = &mut groups[i].1;
                if !names.contains(&name) {
                    names.push(name);
                }
            }
            None => {
                index.insert(key, groups.len());
                groups.push((key, vec![name]));
            }
        }
    }
    if groups.is_empty() {
        return None;
    }

    let mut out = String::new();
    for ((cmd, pid, user), names) in groups.iter().take(LSOF_ROWS) {
        out.push_str(&format!(
            "{} {} ({}): {}\n",
            cmd,
            pid,
            user,
            names.join(", ")
        ));
    }
    if groups.len() > LSOF_ROWS {
        out.push_str(&format!("… +{} more processes\n", groups.len() - LSOF_ROWS));
    }
    out.push_str(&format!(
        "{} processes, {} file descriptors\n",
        groups.len(),
        fds
    ));
    Some(out)
}

/// Identical command lines collapsed into one row carrying every pid.
/// None for bare `pgrep` output (pids only), which is already minimal.
fn compress_pgrep(input: &str) -> Option<String> {
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for line in input.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let (pid, cmd) = line.trim().split_once(char::is_whitespace)?;
        pid.parse::<u32>().ok()?;
        let short = short_cmd(cmd);
        match groups.iter_mut().find(|(c, _)| *c == short) {
            Some((_, pids)) => pids.push(pid.to_string()),
            None => groups.push((short, vec![pid.to_string()])),
        }
    }
    if groups.is_empty() {
        return None;
    }

    let total: usize = groups.iter().map(|(_, p)| p.len()).sum();
    let mut out = String::new();
    for (cmd, pids) in groups.iter().take(PGREP_ROWS) {
        out.push_str(&format!("{}  {}\n", pids.join(","), cmd));
    }
    if groups.len() > PGREP_ROWS {
        out.push_str(&format!("… +{} more\n", groups.len() - PGREP_ROWS));
    }
    out.push_str(&format!("{} processes\n", total));
    Some(out)
}

impl ParseHandler {
    pub(crate) fn handle_du(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        Self::emit_compressed(&input, compress_du(&input), "du", ctx)
    }

    pub(crate) fn handle_lsof(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        Self::emit_compressed(&input, compress_lsof(&input), "lsof", ctx)
    }

    pub(crate) fn handle_pgrep(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        Self::emit_compressed(&input, compress_pgrep(&input), "pgrep", ctx)
    }
}

#[cfg(test)]
#[path = "sysinfo_tests.rs"]
mod tests;
