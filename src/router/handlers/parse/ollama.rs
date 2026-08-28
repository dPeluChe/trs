//! `ollama` output compression, for the two shapes it actually produces.
//!
//! `list` / `ps` are padded tables carrying an ID column that agents never
//! reference: they address models by name, and the 12-hex digest is only
//! useful to `ollama rm`. Dropping it plus the alignment padding is most of
//! the win.
//!
//! `pull` prints one redrawn progress bar per layer. On a real download that
//! is hundreds of carriage-return repaints of the same five lines; what
//! survives is the layer count, the total size and the final verdict.
//!
//! Anything else (`show`, `run`, `serve`) passes through: their output is
//! model text or free-form, and guessing at it is how a parser starts lying.

use super::super::common::{strip_ansi_codes, CommandContext, CommandResult, CommandStats};
use super::ParseHandler;

/// Columns worth keeping from a `list` / `ps` table, by header name.
const KEEP_COLUMNS: &[&str] = &["NAME", "SIZE", "MODIFIED", "PROCESSOR", "CONTEXT", "UNTIL"];

/// Split a header row into `(name, start_offset)` pairs. Column values are
/// aligned under their header, so the offsets are what slices the rows.
fn columns(header: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let mut idx = 0;
    while idx < header.len() {
        let rest = &header[idx..];
        let lead = rest.len() - rest.trim_start().len();
        let start = idx + lead;
        let word: String = header[start..]
            .chars()
            .take_while(|c| !c.is_whitespace())
            .collect();
        if word.is_empty() {
            break;
        }
        idx = start + word.len();
        out.push((word, start));
    }
    out
}

/// A `list` / `ps` table with the unused columns and the padding removed.
fn compress_table(input: &str) -> Option<String> {
    let mut lines = input.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next()?;
    if !header.trim_start().starts_with("NAME") {
        return None;
    }
    let cols = columns(header);
    let keep: Vec<usize> = (0..cols.len())
        .filter(|i| KEEP_COLUMNS.contains(&cols[*i].0.as_str()))
        .collect();
    if keep.is_empty() {
        return None;
    }

    let cell = |line: &str, i: usize| -> String {
        let start = cols[i].1;
        let end = cols.get(i + 1).map(|c| c.1).unwrap_or(line.len());
        line.get(start..end.min(line.len()))
            .unwrap_or("")
            .trim()
            .to_string()
    };

    let mut rows = 0usize;
    let mut out = String::new();
    out.push_str(
        &keep
            .iter()
            .map(|i| cols[*i].0.clone())
            .collect::<Vec<_>>()
            .join("  "),
    );
    out.push('\n');
    for line in lines {
        let cells: Vec<String> = keep.iter().map(|i| cell(line, *i)).collect();
        if cells.iter().all(String::is_empty) {
            continue;
        }
        rows += 1;
        out.push_str(&cells.join("  "));
        out.push('\n');
    }
    // A header with no rows is already minimal (`ollama ps` with nothing up).
    if rows == 0 {
        return None;
    }
    out.push_str(&format!("{} models\n", rows));
    Some(out)
}

/// `pull` progress folded into the layer count, the total, and the verdict.
fn compress_pull(input: &str) -> Option<String> {
    // Progress is redrawn with carriage returns; each repaint is a segment.
    let flat = input.replace('\r', "\n");
    let mut layers: Vec<String> = Vec::new();
    let mut verdict = None;
    let mut saw_pull = false;
    for line in flat.lines() {
        let t = strip_ansi_codes(line);
        let t = t.trim();
        if t.starts_with("pulling manifest") {
            saw_pull = true;
        } else if let Some(rest) = t.strip_prefix("pulling ") {
            saw_pull = true;
            // `pulling <digest>: 100% ▕...▏ 4.9 GB`
            if let Some((digest, tail)) = rest.split_once(':') {
                // The size is the trailing `4.9 GB`, two tokens: taking only
                // the last one leaves the unit with no number attached.
                let mut back = tail.split_whitespace().rev();
                let unit = back.next().unwrap_or("");
                let size = match back.next() {
                    Some(n) if n.starts_with(|c: char| c.is_ascii_digit()) => {
                        format!("{n} {unit}")
                    }
                    _ => unit.to_string(),
                };
                let entry = format!("{} {}", digest.trim(), size);
                if !layers.contains(&entry) {
                    layers.push(entry);
                }
            }
        } else if matches!(t, "success" | "error") || t.starts_with("Error") {
            verdict = Some(t.to_string());
        }
    }
    if !saw_pull {
        return None;
    }
    let mut out = String::new();
    out.push_str(&format!("pulled {} layer(s)\n", layers.len()));
    for l in &layers {
        out.push_str(&format!("  {}\n", l));
    }
    out.push_str(&format!("{}\n", verdict.as_deref().unwrap_or("incomplete")));
    Some(out)
}

impl ParseHandler {
    pub(crate) fn handle_ollama(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let input = Self::read_input(file)?;
        let compressed = compress_pull(&input).or_else(|| compress_table(&input));
        let (out, reducer) = match compressed {
            Some(c) if c.len() < input.len() => (c, "ollama"),
            _ => (input.clone(), "ollama-passthrough"),
        };
        crate::parse_out::emit(&out);
        if ctx.stats {
            CommandStats::new()
                .with_reducer(reducer)
                .with_input_bytes(input.len())
                .with_output_bytes(out.len())
                .print();
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "ollama_tests.rs"]
mod tests;
