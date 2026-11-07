//! Yes/no prompts. No answer at all (closed stdin, `--json`) is not a "no":
//! the command did nothing, so it exits 1 and says how to skip the prompt.

use std::io::{self, BufRead, Write};

/// Whether the user typed `y`/`yes`. Exits 1 when stdin gives no answer.
pub(crate) fn confirm(question: &str, skip_hint: &str) -> bool {
    print!("{question} [y/N] ");
    let _ = io::stdout().flush();
    let line = answer().unwrap_or_else(|| no_answer(skip_hint));
    matches!(line.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

/// One line from stdin, or None when there is none to read.
pub(crate) fn answer() -> Option<String> {
    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(n) if n > 0 => Some(line),
        _ => None,
    }
}

pub(crate) fn no_answer(skip_hint: &str) -> ! {
    println!();
    eprintln!("trs: no answer to the prompt, nothing done; {skip_hint}");
    std::process::exit(1)
}
