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
