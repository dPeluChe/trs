//! Fast-path intercepts for `cat`, `head`, and `sed -n X,Yp`.
//!
//! Agents frequently use these to read file content; intercepting before
//! subprocess spawn applies filter_minimal for 30–50% token reduction on
//! code files vs the 0% from plain passthrough.

use crate::router::handlers::read::{detect_language, filter_minimal, Language};
use crate::router::CommandContext;
use std::path::Path;
use std::time::Instant;

/// Try to intercept `cat [FILE...]` (no flags).
/// Returns true if handled.
pub(crate) fn try_cat(args: &[String], _ctx: &CommandContext) -> bool {
    if args.is_empty() {
        return false;
    }

    let mut files: Vec<&str> = Vec::new();
    let mut past_sep = false;

    for arg in args {
        if !past_sep && arg == "--" {
            past_sep = true;
            continue;
        }
        if !past_sep && arg.starts_with('-') {
            return false; // has flags (-n, -A, -e, etc.)
        }
        if arg == "-" {
            return false; // stdin
        }
        files.push(arg.as_str());
    }

    if files.is_empty() {
        return false;
    }

    // All args must be regular files (not dirs, pipes, /dev/*)
    if !files.iter().all(|f| Path::new(f).is_file()) {
        return false;
    }

    let timer = Instant::now();
    let mut total_in = 0usize;
    let mut output = String::new();

    for file_path in &files {
        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        total_in += content.len();

        let p = Path::new(file_path).to_path_buf();
        let lang = detect_language(&p);
        let filtered = filtered_or_raw(filter_minimal(&content, lang), content, lang);

        if files.len() > 1 {
            output.push_str(&format!("==> {} <==\n", file_path));
        }
        output.push_str(&filtered);
        if !filtered.ends_with('\n') {
            output.push('\n');
        }
    }

    print!("{}", output);

    let ms = timer.elapsed().as_millis() as u64;
    crate::tracker::log_execution(
        &format!("cat {}", args.join(" ")),
        total_in,
        output.len(),
        ms,
    );

    true
}

/// Try to intercept `head [-n N | -N] FILE`.
/// Returns true if handled.
pub(crate) fn try_head(args: &[String], _ctx: &CommandContext) -> bool {
    let Some((limit, file_path)) = parse_head_args(args) else {
        return false;
    };

    let path = Path::new(&file_path);
    if !path.is_file() {
        return false;
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let in_bytes = content.len();
    let timer = Instant::now();

    let all_lines: Vec<&str> = content.lines().collect();
    let hi = limit.min(all_lines.len());
    let sliced = if hi > 0 {
        all_lines[..hi].join("\n") + "\n"
    } else {
        String::new()
    };

    let lang = detect_language(&path.to_path_buf());
    let filtered = filtered_or_raw(filter_minimal(&sliced, lang), sliced, lang);

    print!("{}", filtered);

    let ms = timer.elapsed().as_millis() as u64;
    crate::tracker::log_execution(
        &format!("head {}", args.join(" ")),
        in_bytes,
        filtered.len(),
        ms,
    );

    true
}

/// Try to intercept `sed -n X,Yp FILE`.
/// Returns true if handled.
pub(crate) fn try_sed(args: &[String], _ctx: &CommandContext) -> bool {
    let Some((start, end, file_path)) = parse_sed_range(args) else {
        return false;
    };

    let path = Path::new(&file_path);
    if !path.is_file() {
        return false;
    }

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let in_bytes = content.len();
    let timer = Instant::now();

    let all_lines: Vec<&str> = content.lines().collect();
    let lo = (start as usize).saturating_sub(1).min(all_lines.len());
    let hi = (end as usize).min(all_lines.len());
    let sliced = if lo < hi {
        all_lines[lo..hi].join("\n") + "\n"
    } else {
        String::new()
    };

    let lang = detect_language(&path.to_path_buf());
    let filtered = filtered_or_raw(filter_minimal(&sliced, lang), sliced, lang);

    print!("{}", filtered);

    let ms = timer.elapsed().as_millis() as u64;
    crate::tracker::log_execution(
        &format!("sed -n {},{} {}", start, end, file_path),
        in_bytes,
        filtered.len(),
        ms,
    );

    true
}

/// Parse `head [-n N | -N | --lines=N] FILE` → (line_count, file).
fn parse_head_args(args: &[String]) -> Option<(usize, String)> {
    let mut limit = 10usize;
    let mut file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "-n" || arg == "--lines" {
            i += 1;
            limit = args.get(i)?.parse().ok()?;
        } else if let Some(n) = arg.strip_prefix("-n") {
            limit = n.parse().ok()?;
        } else if let Some(n) = arg.strip_prefix("--lines=") {
            limit = n.parse().ok()?;
        } else if let Some(rest) = arg.strip_prefix('-') {
            // -N shorthand (e.g., -5)
            limit = rest.parse().ok()?;
        } else if file.is_none() {
            file = Some(arg.to_string());
        } else {
            return None; // multiple files, bail
        }
        i += 1;
    }

    Some((limit, file?))
}

/// Parse `sed [-n] X,Yp FILE` → (start, end, file). None if pattern doesn't match.
fn parse_sed_range(args: &[String]) -> Option<(u32, u32, String)> {
    let mut n_flag = false;
    let mut range: Option<(u32, u32)> = None;
    let mut file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();

        if arg == "-n" {
            n_flag = true;
        } else if let Some(expr) = arg.strip_prefix("-n") {
            // -nX,Yp combined form
            n_flag = true;
            range = Some(parse_range_expr(expr)?);
        } else if arg.starts_with('-') {
            return None; // unknown flag → bail
        } else if range.is_none() {
            range = Some(parse_range_expr(arg)?);
        } else if file.is_none() {
            file = Some(arg.to_string());
        } else {
            return None; // unexpected extra arg
        }

        i += 1;
    }

    if !n_flag {
        return None;
    }
    let (start, end) = range?;
    let file = file?;
    Some((start, end, file))
}

/// Return `filtered` if it has meaningful content; otherwise fall back to `raw`.
/// Prevents filter_minimal from returning empty output when a slice is all comments.
fn filtered_or_raw(filtered: String, raw: String, lang: Language) -> String {
    if lang == Language::Data {
        return raw;
    }
    if filtered.trim().is_empty() {
        raw
    } else {
        filtered
    }
}

/// Parse "X,Yp" → (X, Y).
fn parse_range_expr(s: &str) -> Option<(u32, u32)> {
    let s = s.strip_suffix('p')?;
    let (a, b) = s.split_once(',')?;
    let start: u32 = a.trim().parse().ok()?;
    let end: u32 = b.trim().parse().ok()?;
    (start > 0 && end >= start).then_some((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_range_basic() {
        assert_eq!(parse_range_expr("10,20p"), Some((10, 20)));
        assert_eq!(parse_range_expr("1,561p"), Some((1, 561)));
        assert_eq!(parse_range_expr("23,41p"), Some((23, 41)));
    }

    #[test]
    fn parse_range_invalid() {
        assert_eq!(parse_range_expr("10,20"), None); // no trailing p
        assert_eq!(parse_range_expr("s/foo/bar/"), None);
        assert_eq!(parse_range_expr("20,10p"), None); // end < start
        assert_eq!(parse_range_expr("0,10p"), None); // start must be > 0
    }

    #[test]
    fn parse_sed_standard() {
        let args = vec!["-n".into(), "23,41p".into(), "file.rs".into()];
        assert_eq!(parse_sed_range(&args), Some((23, 41, "file.rs".into())));
    }

    #[test]
    fn parse_sed_combined_flag() {
        let args = vec!["-n23,41p".into(), "file.rs".into()];
        assert_eq!(parse_sed_range(&args), Some((23, 41, "file.rs".into())));
    }

    #[test]
    fn parse_sed_no_n_flag() {
        let args = vec!["23,41p".into(), "file.rs".into()];
        assert_eq!(parse_sed_range(&args), None);
    }

    #[test]
    fn parse_sed_inplace_edit() {
        let args = vec!["-i.bak".into(), "1,3d".into(), "file.rs".into()];
        assert_eq!(parse_sed_range(&args), None); // -i flag → bail
    }

    #[test]
    fn parse_sed_substitution() {
        let args = vec!["-n".into(), "s/foo/bar/".into(), "file.rs".into()];
        assert_eq!(parse_sed_range(&args), None); // non-range expression
    }

    #[test]
    fn parse_head_long_form() {
        assert_eq!(
            parse_head_args(&["-n".into(), "20".into(), "file.rs".into()]),
            Some((20, "file.rs".into()))
        );
    }

    #[test]
    fn parse_head_short_form() {
        assert_eq!(
            parse_head_args(&["-20".into(), "file.rs".into()]),
            Some((20, "file.rs".into()))
        );
    }

    #[test]
    fn parse_head_default() {
        assert_eq!(
            parse_head_args(&["file.rs".into()]),
            Some((10, "file.rs".into()))
        );
    }
}
