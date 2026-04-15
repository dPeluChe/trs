use std::process::Command;

use super::format::{extract_section, format_bytes};

/// List available Ollama models.
pub fn list_ollama_models() {
    match get_ollama_models() {
        Some(models) if !models.is_empty() => {
            eprintln!("Ollama models available:");
            for (name, size, family) in &models {
                eprintln!("  {} ({}, {})", name, size, family);
            }
        }
        _ => {
            eprintln!("Ollama not running at localhost:11434");
            eprintln!("Start with: ollama serve");
        }
    }
}

/// Get list of Ollama models: (name, size_display, family).
fn get_ollama_models() -> Option<Vec<(String, String, String)>> {
    let output = Command::new("curl")
        .args(["-s", "http://localhost:11434/api/tags"])
        .output()
        .ok()?;

    let response: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let models = response.get("models")?.as_array()?;

    let mut result: Vec<(String, String, String)> = Vec::new();
    for model in models {
        let name = model.get("name")?.as_str()?.to_string();
        let size = model
            .get("details")
            .and_then(|d| d.get("parameter_size"))
            .and_then(|s| s.as_str())
            .unwrap_or("?")
            .to_string();
        let family = model
            .get("details")
            .and_then(|d| d.get("family"))
            .and_then(|s| s.as_str())
            .unwrap_or("?")
            .to_string();
        // Skip embedding models
        if name.contains("embed") || name.contains("nomic") {
            continue;
        }
        result.push((name, size, family));
    }

    Some(result)
}

/// Pick the best available local model (prefer larger, local over cloud).
fn pick_default_model() -> Option<String> {
    let models = get_ollama_models()?;
    // Prefer local models (no :cloud suffix) with largest param count
    let local: Vec<&(String, String, String)> = models
        .iter()
        .filter(|(name, _, _)| !name.contains(":cloud"))
        .collect();

    if let Some(best) = local.first() {
        return Some(best.0.clone());
    }
    // Fallback to any model
    models.first().map(|(name, _, _)| name.clone())
}

/// Send digest to Ollama for LLM-formatted summary.
pub(super) fn ollama_format(digest: &str, model: &str) -> Option<String> {
    // Resolve model: "auto" picks the best available
    let model = if model == "auto" {
        match pick_default_model() {
            Some(m) => {
                eprintln!("trs ingest: using Ollama model: {}", m);
                m
            }
            None => {
                eprintln!(
                    "trs ingest: no Ollama models found. Install one with: ollama pull llama3.1"
                );
                return None;
            }
        }
    } else {
        model.to_string()
    };

    // Verify Ollama is running and model exists
    let models = get_ollama_models();
    if models.is_none() {
        eprintln!("trs ingest: Ollama not running at localhost:11434");
        eprintln!("  Start with: ollama serve");
        return None;
    }

    let model_exists = models
        .as_ref()
        .map(|m| {
            m.iter()
                .any(|(n, _, _)| n == &model || n.starts_with(&format!("{}:", model)))
        })
        .unwrap_or(false);

    if !model_exists {
        eprintln!("trs ingest: model '{}' not found in Ollama", model);
        list_ollama_models();
        return None;
    }

    // Extract README from digest if present (for better Ollama context)
    let readme_content = extract_section(digest, "README.md")
        .or_else(|| extract_section(digest, "readme.md"))
        .unwrap_or_default();

    // Truncate digest to fit model context (conservative 24k chars)
    let max_chars = 24_000;
    let input = if digest.len() > max_chars {
        eprintln!(
            "trs ingest: digest truncated to {} for Ollama context",
            format_bytes(max_chars)
        );
        // Put README first, then as much of the rest as fits
        let readme_section = if !readme_content.is_empty() {
            format!("## README.md\n\n{}\n\n---\n\n", readme_content)
        } else {
            String::new()
        };
        let remaining = max_chars.saturating_sub(readme_section.len());
        let mut combined = readme_section;
        // Find safe char boundary
        let mut cut = remaining.min(digest.len());
        while cut > 0 && !digest.is_char_boundary(cut) {
            cut -= 1;
        }
        combined.push_str(&digest[..cut]);
        combined
    } else {
        digest.to_string()
    };
    let input = &input;

    eprintln!(
        "trs ingest: generating summary with {} ({})...",
        model,
        format_bytes(input.len())
    );

    let prompt = format!(
        "Analyze this codebase digest and produce a structured markdown summary.\n\n\
         Output EXACTLY this format:\n\n\
         ## Overview\n\
         [1-2 sentences: what this project does]\n\n\
         ## Tech Stack\n\
         [bullet list: language, framework, database, key dependencies]\n\n\
         ## Architecture\n\
         [bullet list: key directories/modules and their responsibility]\n\n\
         ## Key Files\n\
         [bullet list: 5-10 most important files and what they do]\n\n\
         ## Entry Points\n\
         [bullet list: main entry files, API routes, CLI commands]\n\n\
         Be concise and factual. No explanations of what markdown is. \
         No filler. Just the structured summary.\n\n---\n\n{}",
        input
    );

    let payload = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "stream": false,
        "options": {
            "temperature": 0.3,
            "num_predict": 2048,
        }
    });

    let output = Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "http://localhost:11434/api/generate",
            "-H",
            "Content-Type: application/json",
            "-d",
            &payload.to_string(),
        ])
        .output()
        .ok()?;

    let response: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    let formatted = response.get("response")?.as_str()?;

    let duration = response
        .get("total_duration")
        .and_then(|d| d.as_u64())
        .map(|ns| ns / 1_000_000_000)
        .unwrap_or(0);

    eprintln!("trs ingest: Ollama completed in {}s", duration);

    // Combine: LLM summary at top, then raw digest
    let mut result = String::new();
    result.push_str(&format!(
        "# Project Summary\n\n> Generated by {} via Ollama\n\n",
        model
    ));
    result.push_str(formatted);
    result.push_str("\n\n---\n\n# Full Digest\n\n");
    result.push_str(digest);

    Some(result)
}
