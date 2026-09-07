use memecoin_os_core::research::ResearchReport;
use serde::Deserialize;
use serde_json::json;

const DEFAULT_BASE: &str = "https://integrate.api.nvidia.com/v1";
const DEFAULT_MODEL: &str = "nvidia/nemotron-3.5-lightning-30b-a3b";

pub fn nvidia_configured() -> bool {
    std::env::var("NVIDIA_API_KEY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false)
}

/// Rewrite the executive summary only. Numbers must already exist in the grounded report.
pub async fn polish(mut report: ResearchReport) -> ResearchReport {
    if !crate::env_flag("FEATURE_AI_RESEARCH", true) || !nvidia_configured() {
        return report;
    }
    let Ok(key) = std::env::var("NVIDIA_API_KEY") else {
        return report;
    };
    let key = key.trim().to_string();
    if key.is_empty() {
        return report;
    }
    let base = std::env::var("NVIDIA_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE.into());
    let models = model_candidates();
    let grounded = json!({
        "executive_summary": report.executive_summary,
        "observations": report.observations.iter().map(|o| &o.observation).collect::<Vec<_>>(),
        "unknown": report.unknown,
        "evidence": report.evidence,
        "limitations": report.limitations,
        "confidence": report.confidence,
    });
    let allow = grounded.to_string();
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("MemeCoinOS/0.1 (grounded-llm)")
        .build()
    {
        Ok(c) => c,
        Err(_) => return report,
    };
    let url = format!("{}/chat/completions", base.trim_end_matches('/'));
    for model in &models {
        let body = json!({
            "model": model,
            "temperature": 0.2,
            "max_tokens": 400,
            "messages": [
                {
                    "role": "system",
                    "content": "You rewrite an ecosystem intelligence executive summary. Use ONLY facts in the JSON. If a field is missing or unknown, say MISSING. No price targets. No guarantees. No EXECUTE. No new numbers, tickers, or contracts."
                },
                {
                    "role": "user",
                    "content": format!("Question: {}\nJSON:\n{}", report.question, allow)
                }
            ]
        });
        let res = client
            .post(&url)
            .bearer_auth(&key)
            .json(&body)
            .send()
            .await;
        let res = match res {
            Ok(r) if r.status().is_success() => r,
            Ok(r) => {
                let stale = r.status().as_u16() == 404 || r.status().as_u16() == 410;
                tracing::warn!(status = %r.status(), model = %model, "nvidia chat non-success");
                if stale {
                    continue;
                }
                return report;
            }
            Err(e) => {
                tracing::warn!(error = %e, "nvidia unreachable — keeping grounded text");
                return report;
            }
        };
        let parsed: ChatResponse = match res.json().await {
            Ok(v) => v,
            Err(_) => continue,
        };
        let Some(text) = parsed
            .choices
            .into_iter()
            .find_map(|c| c.message.and_then(|m| m.content))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        else {
            continue;
        };
        if !numbers_allowed(&text, &allow) {
            tracing::warn!(model = %model, "nvidia output introduced numbers — discarded");
            continue;
        }
        report.executive_summary = text.clone();
        report.model_id = format!("research-agent/v1+{model}");
        report.markdown = format!(
            "{}\n\n## Narrative (NVIDIA, grounded)\n\n{}\n",
            report.markdown, text
        );
        return report;
    }
    report
}

fn model_candidates() -> Vec<String> {
    let mut out = Vec::new();
    let push = |out: &mut Vec<String>, raw: &str| {
        for part in raw.split(',') {
            let id = part.trim();
            if !id.is_empty() && !out.iter().any(|e| e == id) {
                out.push(id.to_string());
            }
        }
    };
    if let Ok(primary) = std::env::var("NVIDIA_MODEL") {
        push(&mut out, &primary);
    }
    if let Ok(list) = std::env::var("NVIDIA_MODELS") {
        push(&mut out, &list);
    }
    if out.is_empty() {
        out.push(DEFAULT_MODEL.into());
    }
    out
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    #[serde(default)]
    choices: Vec<Choice>,
}
#[derive(Debug, Deserialize)]
struct Choice {
    message: Option<Msg>,
}
#[derive(Debug, Deserialize)]
struct Msg {
    content: Option<String>,
}

fn numbers_allowed(output: &str, allow: &str) -> bool {
    for n in extract_numbers(output) {
        if !allow.contains(&n) {
            return false;
        }
    }
    true
}

fn extract_numbers(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in s.chars() {
        if ch.is_ascii_digit() || (ch == '.' && !cur.is_empty()) {
            cur.push(ch);
        } else if !cur.is_empty() {
            if cur.chars().any(|c| c.is_ascii_digit()) {
                out.push(cur.clone());
            }
            cur.clear();
        }
    }
    if !cur.is_empty() && cur.chars().any(|c| c.is_ascii_digit()) {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invented_price() {
        let allow = r#"{"executive_summary":"health 41","confidence":0.4}"#;
        assert!(!numbers_allowed("Price will be 12.5", allow));
        assert!(numbers_allowed("health 41 with confidence 0.4", allow));
    }
}
