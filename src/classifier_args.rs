//! Argument preprocessing for the classifier: `tail -N` shorthand, git global
//! option stripping, and structured-output flag detection. Kept separate from
//! the `classify_command` dispatch so each stays focused.

/// Expand `tail -N` shorthand to `tail -n N`.
pub(crate) fn preprocess_tail_args(args: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        // Check if we're in a tail command context
        if i > 0 && (args[i - 1] == "tail" || is_after_tail_subcommand(args, i)) {
            // Check if this is a -N argument (negative number like -5, -20, etc.)
            if let Some(number) = arg.strip_prefix('-') {
                if let Ok(n) = number.parse::<usize>() {
                    // Transform -N to -n N
                    result.push("-n".to_string());
                    result.push(n.to_string());
                    i += 1;
                    continue;
                }
            }
        }

        result.push(arg.clone());
        i += 1;
    }

    result
}

/// Check if the current position is after a tail subcommand (accounting for global flags).
pub(crate) fn is_after_tail_subcommand(args: &[String], pos: usize) -> bool {
    // Look backwards to find if we have a "tail" command
    for j in (0..pos).rev() {
        if args[j] == "tail" {
            return true;
        }
        // If we hit another subcommand, stop looking
        if j > 0 && !args[j].starts_with('-') && args[j - 1].starts_with('-') {
            break;
        }
    }
    false
}

/// Strip git global options that appear before the subcommand.
/// Returns the args with global options removed so the subcommand can be detected.
/// Global options: -C <path>, -c <key=val>, --git-dir=<path>, --work-tree=<path>,
/// --no-pager, --no-optional-locks, --bare, --literal-pathspecs
pub(crate) fn strip_git_global_opts(args: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            // Options that consume the next argument
            "-C" | "-c" | "--git-dir" | "--work-tree" => {
                i += 2; // skip flag + value
                continue;
            }
            // Options with = syntax
            a if a.starts_with("--git-dir=")
                || a.starts_with("--work-tree=")
                || a.starts_with("-c=") =>
            {
                i += 1;
                continue;
            }
            // Standalone flags
            "--no-pager"
            | "--no-optional-locks"
            | "--bare"
            | "--literal-pathspecs"
            | "--no-replace-objects"
            | "--no-lazy-fetch" => {
                i += 1;
                continue;
            }
            _ => {
                result.push(args[i].clone());
                i += 1;
            }
        }
    }
    result
}

/// Extract the inner command from `bash -c "<script>"` / `sh -c` / `zsh -c`
/// when the script is a SINGLE simple command. Compound scripts (`;`, `|`,
/// `&&`, redirects, substitutions, quotes) return None — their output is
/// mixed, so no single parser applies and generic compression is correct.
/// Returns the inner argv (first element = binary).
pub(crate) fn unwrap_shell_c(args: &[String]) -> Option<Vec<String>> {
    // Accept `-c <script>` and fused single-flag forms (`-lc`, `-ec`);
    // nothing after the script (positional $0 args change semantics).
    let script = match args {
        [flag, script] if flag == "-c" || flag == "-lc" || flag == "-ec" => script,
        _ => return None,
    };
    if script.contains([
        ';', '|', '&', '<', '>', '`', '$', '(', ')', '{', '}', '\'', '"', '\\', '\n',
    ]) {
        return None;
    }
    let tokens: Vec<String> = script.split_whitespace().map(String::from).collect();
    if tokens.is_empty() {
        return None;
    }
    Some(tokens)
}

/// Unwrap `timeout [OPTS] DURATION CMD [ARGS…]` to the inner command argv
/// so it reaches the right parser, like `bash -c`. Skips leading flags (and
/// the value of `-s/--signal/-k/--kill-after` when given as a separate
/// token), then the DURATION, returning `[CMD, ARGS…]`. Returns None when
/// the shape doesn't match (no digit-leading duration, or no inner command).
pub(crate) fn unwrap_timeout(args: &[String]) -> Option<Vec<String>> {
    let mut i = 0;
    while let Some(a) = args.get(i).map(|s| s.as_str()) {
        if !a.starts_with('-') {
            break;
        }
        // Options that take a separate value token.
        i += if matches!(a, "-s" | "--signal" | "-k" | "--kill-after") {
            2
        } else {
            1
        };
    }
    // args[i] is DURATION (e.g. `10`, `5s`, `1.5m`); guard on a leading
    // digit so `timeout --help` and friends stay passthrough.
    let dur = args.get(i)?;
    if !dur.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return None;
    }
    let inner = args.get(i + 1..)?;
    if inner.is_empty() {
        return None;
    }
    Some(inner.to_vec())
}

/// Check if the command args contain flags that indicate structured output.
/// When the user explicitly requests JSON/structured output, we should passthrough.
pub(crate) fn has_structured_output_flag(args: &[String]) -> bool {
    args.iter().any(|a| {
        let s = a.as_str();
        s == "--json"
            || s == "--porcelain"
            || s == "--format=json"
            || s == "--output=json"
            || s == "-o=json"
            || s == "--format" && args.iter().any(|b| b == "json")
            || s.starts_with("--format=json")
            || s.starts_with("--output=json")
    })
}

/// What one of trs's own flags asks for.
#[derive(Clone, Copy)]
enum Flag {
    Format(crate::OutputFormat),
    Stats,
}

/// trs's own output flags, accepted after an external command
/// (`trs git status --json`).
const FLAGS: &[(&str, Flag)] = &[
    ("--json", Flag::Format(crate::OutputFormat::Json)),
    ("--csv", Flag::Format(crate::OutputFormat::Csv)),
    ("--tsv", Flag::Format(crate::OutputFormat::Tsv)),
    ("--agent", Flag::Format(crate::OutputFormat::Agent)),
    ("--compact", Flag::Format(crate::OutputFormat::Compact)),
    ("--raw", Flag::Format(crate::OutputFormat::Raw)),
    ("--stats", Flag::Stats),
];

fn trs_flag(arg: &str) -> Option<Flag> {
    FLAGS.iter().find(|(name, _)| *name == arg).map(|(_, f)| *f)
}

/// Whether the program itself defines `flag`, so trs must hand it over
/// instead of reading it as its own (`gh run list --json id`, `git log --raw`).
/// `args` are the command's arguments without trs's flags.
pub(crate) fn child_owns_format_flag(cmd: &str, args: &[String], flag: &str) -> bool {
    match (crate::text_util::cmd_basename(cmd), flag) {
        (
            "gh" | "glab" | "npm" | "pnpm" | "yarn" | "bun" | "brew" | "cargo" | "rg" | "jest"
            | "vitest" | "webpack",
            "--json",
        ) => true,
        ("git", "--raw") => matches!(
            strip_git_global_opts(args).first().map(String::as_str),
            Some(
                "diff" | "log" | "show" | "diff-tree" | "diff-index" | "diff-files" | "whatchanged"
            )
        ),
        ("kubectl" | "oc" | "curl" | "mysql" | "mariadb", "--raw") => true,
        ("rg" | "rsync" | "eslint" | "webpack", "--stats") => true,
        ("psql", "--csv") => true,
        _ => false,
    }
}

/// Splits `trs <cmd> ...` (argv without the program name) into the arguments
/// for the command and trs's own output flags.
///
/// A flag after the command is trs's only when trs has a formatter for that
/// command and the command does not take the flag itself. Everything else is
/// the command's: dropping it changes what the agent asked for (`gh run list
/// --json id` lost its `--json`, `git log --raw` its raw format). Before the
/// command (`trs --json git status`) a flag is always trs's. After a bare
/// `--` nothing is trs's.
pub(crate) fn split_format_flags(
    args: &[String],
) -> (Vec<String>, Option<crate::OutputFormat>, bool) {
    // Common case: no trs flag, nothing to classify or copy around.
    if !args.iter().any(|a| trs_flag(a).is_some()) {
        return (args.to_vec(), None, false);
    }
    let cmd = args.first().map(String::as_str).unwrap_or("");
    let end = args.iter().position(|a| a == "--").unwrap_or(args.len());
    let plain: Vec<String> = args[1..]
        .iter()
        .filter(|a| trs_flag(a).is_none())
        .cloned()
        .collect();
    let has_formatter = crate::classifier::classify_command(cmd, &plain).is_some();

    let mut kept = Vec::with_capacity(args.len());
    let (mut format, mut stats) = (None, false);
    for (i, arg) in args.iter().enumerate() {
        let ours = (1..end)
            .contains(&i)
            .then(|| trs_flag(arg))
            .flatten()
            .filter(|_| has_formatter && !child_owns_format_flag(cmd, &plain, arg));
        match ours {
            Some(Flag::Format(f)) => format = Some(f),
            Some(Flag::Stats) => stats = true,
            None => kept.push(arg.clone()),
        }
    }
    (kept, format, stats)
}

#[cfg(test)]
#[path = "classifier_args_tests.rs"]
mod tests;
