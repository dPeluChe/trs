//! `trs init --show` and the combined status+usage screen. Split from
//! init.rs so the main module stays focused on dispatch and detection.

use crate::init::{check_tool, AiTool};

/// Print current hook installation status.
///
/// Markers:
/// - `+` configured with trs
/// - `•` installed on system, not configured
/// - `-` not detected (config still possible for future installs)
pub(crate) fn show_status() {
    println!("trs init: hook status\n");

    let tools = AiTool::all_tools();
    let name_width = tools.iter().map(|t| t.name().len()).max().unwrap_or(0);

    let mut configured = 0;
    let mut detected_total = 0;
    for tool in &tools {
        let is_configured = check_tool(tool);
        let is_detected = tool.detect_installed();
        let marker = if is_configured {
            "+"
        } else if is_detected {
            "•"
        } else {
            "-"
        };
        let status = if is_configured || is_detected {
            tool.target_label()
        } else {
            "not detected on this system"
        };
        println!(
            "  {} {:<width$}  {}",
            marker,
            tool.name(),
            status,
            width = name_width
        );
        if is_configured {
            configured += 1;
        }
        if is_detected {
            detected_total += 1;
        }
    }
    println!(
        "\n{}/{} configured  ({} detected on system)",
        configured,
        tools.len(),
        detected_total
    );
}

pub(crate) fn show_status_and_usage() {
    show_status();
    println!();
    println!("Usage:");
    println!("  trs init <tool> [--global]      install for a specific tool");
    println!("  trs init --all [--global]       install for all detected tools");
    println!("  trs init --show                 show this status");
    println!();
    println!("Collision handling:");
    println!("  trs init scans the target config for hooks from another compressor");
    println!("  tool (rtk, token-optimizer) before writing. Running two compressors");
    println!("  on the same command risks double-compression, garbled output that");
    println!("  looks successful to the hook layer. By default trs aborts when it");
    println!("  finds a collision.");
    println!();
    println!("  --replace    clean up the other tool's hook before installing trs");
    println!("  --force      install trs alongside anyway (risky, keeps both active)");
    println!();
    println!("Refreshing hooks:");
    println!("  Templates may change between releases. When all agents already show");
    println!("  as configured, re-run with --force to overwrite with the current");
    println!("  template. The config merge preserves any user-added hooks that");
    println!("  don't reference trs.");
    println!();
    println!("More: https://github.com/dPeluChe/trs/blob/main/docs/features/init.md");
}
