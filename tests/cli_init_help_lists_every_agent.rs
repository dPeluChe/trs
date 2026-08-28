use assert_cmd::Command;

/// Every agent trs can install must be named in `trs init --help`.
///
/// This drifted once already: the list said "claude, gemini, cursor, codex,
/// opencode, kilo" while `ai_tool.rs` had grown to 16 tools, so ten
/// integrations were invisible to anyone reading the help. The list lives in
/// a clap doc comment, which cannot call `AiTool::all_names()`, so this test
/// is what keeps the two in step: add a tool without updating the help and
/// it fails here.
#[test]
fn init_help_names_every_supported_agent() {
    let out = Command::cargo_bin("trs")
        .unwrap()
        .args(["init", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&out.stdout);

    // Mirrors AiTool::all_names(), pinned in src/ai_tool_tests.rs.
    const AGENTS: &[&str] = &[
        "claude",
        "gemini",
        "cursor",
        "codex",
        "opencode",
        "kilo",
        "antigravity",
        "agy",
        "droid",
        "devin",
        "pi",
        "vscode",
        "openclaw",
        "hermes",
        "zed",
        "devin-cli",
    ];
    for agent in AGENTS {
        assert!(
            help.contains(agent),
            "`trs init --help` never mentions `{agent}`. If a tool was added \
             to TOOLS in ai_tool.rs, update the doc comment on the Init \
             command in commands.rs to match AiTool::all_names()."
        );
    }
}
