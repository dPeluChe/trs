use super::super::common::{CommandContext, CommandResult, CommandStats};
use super::ParseHandler;
use crate::OutputFormat;

impl ParseHandler {
    /// Parse `gh run list` output.
    pub(crate) fn handle_gh_run(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let raw_input = Self::read_input_raw(file)?;
        let input = super::super::common::strip_emojis(&raw_input);
        let input_bytes = raw_input.len();
        let mut runs: Vec<serde_json::Value> = Vec::new();

        let raw_lines: Vec<&str> = raw_input.lines().collect();
        let clean_lines: Vec<&str> = input.lines().collect();

        for (i, line) in clean_lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let raw_line = raw_lines.get(i).unwrap_or(&"");

            if trimmed.contains('\t') {
                // TSV: status\tconclusion\tname\tdisplay_title\tbranch\tevent\tid\telapsed\tdate
                let fields: Vec<&str> = trimmed.split('\t').collect();
                if fields.len() >= 3 {
                    let status_text = fields[0].trim().to_lowercase();
                    let conclusion = fields[1].trim().to_lowercase();
                    let name = fields[2].trim();
                    let event = fields.get(5).map(|s| s.trim()).unwrap_or("");
                    let status = if conclusion == "success" {
                        "success"
                    } else if conclusion == "failure" {
                        "failure"
                    } else if status_text == "in_progress" {
                        "in_progress"
                    } else if conclusion == "cancelled" {
                        "cancelled"
                    } else {
                        &status_text
                    };
                    runs.push(serde_json::json!({"name": name, "event": event, "status": status}));
                }
            } else {
                if trimmed.starts_with("Workflow") || trimmed.starts_with("Showing") {
                    continue;
                }

                if let Some(bracket_start) = trimmed.rfind('[') {
                    let name = trimmed[..bracket_start].trim();
                    let id = trimmed[bracket_start + 1..].trim_end_matches(']').trim();

                    let status = if raw_line.contains('\u{2705}')
                        || raw_line.contains("success")
                        || raw_line.contains("completed")
                    {
                        "success"
                    } else if raw_line.contains('\u{274C}')
                        || raw_line.contains("failure")
                        || raw_line.contains("failed")
                    {
                        "failure"
                    } else if raw_line.contains("in_progress")
                        || raw_line.contains("queued")
                        || raw_line.contains('\u{1F7E1}')
                    {
                        "in_progress"
                    } else if raw_line.contains('\u{1F534}') || raw_line.contains("cancelled") {
                        "cancelled"
                    } else {
                        "unknown"
                    };

                    if !name.is_empty() {
                        runs.push(serde_json::json!({"name": name, "id": id, "status": status}));
                    }
                }
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => {
                serde_json::json!({"runs": runs, "count": runs.len()}).to_string()
            }
            _ => {
                if runs.is_empty() {
                    "no workflow runs\n".to_string()
                } else {
                    let mut out = format!("runs: {}\n", runs.len());
                    for run in &runs {
                        let marker = match run["status"].as_str().unwrap_or("") {
                            "success" => "+",
                            "failure" => "-",
                            "in_progress" => "~",
                            _ => "?",
                        };
                        let name = Self::truncate_str(run["name"].as_str().unwrap_or(""), 50);
                        let event = run["event"].as_str().unwrap_or("");
                        if !event.is_empty() {
                            out.push_str(&format!("  {} {} ({})\n", marker, name, event));
                        } else {
                            out.push_str(&format!("  {} {}\n", marker, name));
                        }
                    }
                    out
                }
            }
        };
        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("gh-run")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .with_items_processed(runs.len())
                .print();
        }
        Ok(())
    }

    /// Parse `gh run view <id>` output.
    /// Keeps run name, conclusion, job summary, annotations, and URL.
    pub(crate) fn handle_gh_run_view(
        file: &Option<std::path::PathBuf>,
        ctx: &CommandContext,
    ) -> CommandResult {
        let raw_input = Self::read_input_raw(file)?;
        let input = super::super::common::strip_emojis(&raw_input);
        let input_bytes = raw_input.len();

        let mut title = String::new();
        let mut conclusion = "unknown";
        let mut jobs_ok: u32 = 0;
        let mut jobs_fail: u32 = 0;
        let mut annotations: Vec<String> = Vec::new();
        let mut url = String::new();
        let mut in_jobs = false;
        let mut in_annotations = false;

        for (raw_line, clean_line) in raw_input.lines().zip(input.lines()) {
            let trimmed = clean_line.trim();
            if trimmed.is_empty() {
                in_jobs = false;
                in_annotations = false;
                continue;
            }

            if trimmed == "JOBS" {
                in_jobs = true;
                continue;
            }
            if trimmed == "ANNOTATIONS" {
                in_annotations = true;
                in_jobs = false;
                continue;
            }

            if trimmed.starts_with("For more information") || trimmed.starts_with("X ") {
                continue;
            }

            if trimmed.starts_with("View this run on GitHub:") {
                url = trimmed
                    .trim_start_matches("View this run on GitHub:")
                    .trim()
                    .to_string();
                continue;
            }

            if in_jobs {
                let ok = raw_line.contains('\u{2705}')
                    || raw_line.contains("success")
                    || raw_line.contains("completed");
                let fail = raw_line.contains('\u{274C}')
                    || raw_line.contains("failure")
                    || raw_line.contains("failed");
                if fail {
                    jobs_fail += 1;
                } else if ok {
                    jobs_ok += 1;
                }
                continue;
            }

            if in_annotations {
                if annotations.len() < 3 {
                    annotations.push(trimmed.to_string());
                }
                continue;
            }

            if title.is_empty() {
                let line_clean = trimmed.trim_start_matches(|c: char| !c.is_alphanumeric());
                title = if let Some(pos) = line_clean.find(": ") {
                    line_clean[pos + 2..].to_string()
                } else {
                    line_clean.to_string()
                };

                conclusion = if raw_line.contains('\u{2705}')
                    || raw_line.contains("success")
                    || raw_line.contains("completed")
                {
                    "success"
                } else if raw_line.contains('\u{274C}')
                    || raw_line.contains("failure")
                    || raw_line.contains("failed")
                {
                    "failure"
                } else if raw_line.contains('\u{1F7E1}')
                    || raw_line.contains("in_progress")
                    || raw_line.contains("queued")
                {
                    "in_progress"
                } else {
                    "unknown"
                };
            }
        }

        let output = match ctx.format {
            OutputFormat::Json => serde_json::json!({
                "title": title,
                "conclusion": conclusion,
                "jobs": {"ok": jobs_ok, "fail": jobs_fail},
                "annotations": annotations,
                "url": url,
            })
            .to_string(),
            _ => {
                let mut out = format!("run: {} ({})\n", Self::truncate_str(&title, 60), conclusion);
                out.push_str(&format!("jobs: {} ok, {} fail\n", jobs_ok, jobs_fail));
                for ann in &annotations {
                    out.push_str(&format!("  ! {}\n", Self::truncate_str(ann, 80)));
                }
                if !url.is_empty() {
                    out.push_str(&format!("url: {}\n", url));
                }
                out
            }
        };

        crate::parse_out::emit(&output);
        if ctx.stats {
            CommandStats::new()
                .with_reducer("gh-run-view")
                .with_input_bytes(input_bytes)
                .with_output_bytes(output.len())
                .print();
        }
        Ok(())
    }
}
