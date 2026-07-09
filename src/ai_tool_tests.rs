use super::*;

#[test]
fn antigravity_aliases_resolve_to_ide() {
    // Back-compat: pre-v0.6.4 users typing `trs init antigravity`
    // land on the IDE variant. Explicit aliases stay explicit.
    assert!(matches!(
        AiTool::from_str("antigravity"),
        Some(AiTool::Antigravity)
    ));
    assert!(matches!(
        AiTool::from_str("antigravity-ide"),
        Some(AiTool::Antigravity)
    ));
    assert!(matches!(
        AiTool::from_str("gravity"),
        Some(AiTool::Antigravity)
    ));
}

#[test]
fn antigravity_cli_aliases() {
    assert!(matches!(
        AiTool::from_str("antigravity-cli"),
        Some(AiTool::AntigravityCLI)
    ));
    // `agy` is the binary name — most likely thing a user will type.
    assert!(matches!(
        AiTool::from_str("agy"),
        Some(AiTool::AntigravityCLI)
    ));
}

#[test]
fn antigravity_variants_are_rules_only() {
    // v0.6.6 revert: agy v1.0.1 doesn't expose user-configurable
    // PreTool hooks. Both Antigravity variants moved to rules-only
    // (like Codex/Windsurf). Regression on this test would mean
    // we accidentally re-installed a non-functional hook.
    assert!(
        AiTool::Antigravity.spec().is_none(),
        "Antigravity IDE must have no HookSpec — rules-only"
    );
    assert!(
        AiTool::AntigravityCLI.spec().is_none(),
        "Antigravity CLI must have no HookSpec — rules-only"
    );
}

#[test]
fn antigravity_target_label_signals_rules_only() {
    // The label drives `trs init --show`; it must read as
    // rules-only so users know auto-rewriting is off, with a hint
    // at the upstream limitation.
    let ide_label = AiTool::Antigravity.target_label();
    let cli_label = AiTool::AntigravityCLI.target_label();
    assert!(ide_label.starts_with("rules → "));
    assert!(cli_label.starts_with("rules → "));
    assert!(ide_label.contains("GEMINI.md"));
    assert!(cli_label.contains("GEMINI.md"));
}

#[test]
fn registry_covers_every_variant_and_has_no_dup_aliases() {
    // identity() panics if a variant is missing from TOOLS.
    for spec in TOOLS {
        assert_eq!(spec.variant.identity().display, spec.display);
    }
    // No alias is claimed by two tools (would make from_str ambiguous).
    let mut seen = std::collections::HashSet::new();
    for spec in TOOLS {
        for a in spec.aliases {
            assert!(seen.insert(*a), "duplicate alias across tools: {a}");
        }
        // cli_name must itself be a valid alias.
        assert!(
            spec.aliases.contains(&spec.cli_name),
            "{} cli_name not in aliases",
            spec.cli_name
        );
    }
    assert_eq!(AiTool::all_tools().len(), TOOLS.len());
}

#[test]
fn all_names_is_the_cli_name_list() {
    // Pins the exact public string `trs uninstall` prints on bad input.
    assert_eq!(
        AiTool::all_names(),
        "claude, gemini, cursor, codex, opencode, kilo, antigravity, agy, droid, devin, pi, vscode, openclaw, hermes, zed, devin-cli"
    );
}

#[test]
fn devin_cli_is_a_hook_not_rules_only() {
    // Regression guard: the CLI is a real PreToolUse hook (unlike the
    // rules-only Devin Desktop). Its `exec` matcher + `--caller devin-cli`
    // must survive template edits.
    assert!(matches!(
        AiTool::from_str("devin-cli"),
        Some(AiTool::DevinCLI)
    ));
    assert!(matches!(AiTool::from_str("dcli"), Some(AiTool::DevinCLI)));
    // Desktop stays rules-only and keeps the bare `devin` alias.
    assert!(matches!(AiTool::from_str("devin"), Some(AiTool::Devin)));
    let spec = AiTool::DevinCLI
        .spec()
        .expect("Devin CLI must have a HookSpec");
    assert_eq!(spec.filename, "config.json");
    assert!(spec.content.contains("\"exec\""));
    assert!(spec.content.contains("trs rewrite --caller devin-cli"));
}

#[test]
fn output_saver_agents_match_registry() {
    // The output_saver AGENTS list is a separate const (it owns its own
    // display order for `--show`/verify). This guards against its
    // display strings drifting from the identity registry: every agent
    // id must resolve to a known tool whose display name matches.
    for agent in crate::output_saver::AGENTS {
        let tool = AiTool::from_str(agent.id)
            .unwrap_or_else(|| panic!("output_saver agent id `{}` unknown to registry", agent.id));
        assert_eq!(
            tool.name(),
            agent.display,
            "display drift for `{}`: registry={}, output_saver={}",
            agent.id,
            tool.name(),
            agent.display
        );
    }
}
