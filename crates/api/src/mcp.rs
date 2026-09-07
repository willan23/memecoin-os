use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use memecoin_os_core::claims;
use memecoin_os_core::ecosystem;
use memecoin_os_core::models::TokenSnapshot;
use memecoin_os_core::twin;
use memecoin_os_core::twin_similarity;
use serde_json::{json, Value};

const PROTOCOL: &str = "2024-11-05";

pub fn manifest() -> Value {
    json!({
        "name": "memecoin-os",
        "version": "0.1.0",
        "protocol": PROTOCOL,
        "transport": ["stdio", "http"],
        "endpoint": "/v1/mcp",
        "execute": false,
        "note": "Read-only Exchange Intelligence MCP. Not a trade signal. VERIFIED ≠ SAFE. Missing sources stay blank.",
        "tools": tools_list()
    })
}

pub async fn get_manifest() -> Json<Value> {
    Json(manifest())
}

pub async fn rpc(
    State(st): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, StatusCode> {
    if body.as_array().is_some() {
        return Ok(Json(json!({
            "jsonrpc": "2.0",
            "id": null,
            "error": { "code": -32600, "message": "batch not supported" }
        })));
    }
    let id = body.get("id").cloned().unwrap_or(Value::Null);
    let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
    if method.starts_with("notifications/") {
        return Ok(Json(json!({ "jsonrpc": "2.0", "id": id, "result": {} })));
    }
    let result = match method {
        "initialize" => json!({
            "protocolVersion": PROTOCOL,
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "memecoin-os", "version": "0.1.0" },
            "instructions": "Read-only ecosystem intelligence. EXECUTE is off. Do not invent MISSING numbers or price targets."
        }),
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tools_list() }),
        "tools/call" => {
            let params = body.get("params").cloned().unwrap_or(json!({}));
            match call_tool(&st, &params).await {
                Ok(v) => v,
                Err(msg) => {
                    return Ok(Json(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32000, "message": msg }
                    })));
                }
            }
        }
        "" => {
            return Ok(Json(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32600, "message": "method required" }
            })));
        }
        other => {
            return Ok(Json(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32601, "message": format!("unknown method {other}") }
            })));
        }
    };
    Ok(Json(json!({ "jsonrpc": "2.0", "id": id, "result": result })))
}

fn tools_list() -> Value {
    json!([
        tool("health", "Process and provider health. Not a trade signal.", json!({ "type": "object", "properties": {} })),
        tool("list_ecosystems", "Catalog of tracked ecosystems.", json!({ "type": "object", "properties": {} })),
        tool("get_ecosystem", "Ecosystem + asset card.", id_schema()),
        tool("get_asset", "Listing-page asset card (health, risk, holders).", id_schema()),
        tool("get_twin", "Digital Twin snapshot.", id_schema()),
        tool("get_risk", "Multi-factor risk. UNKNOWN when evidence is missing.", id_schema()),
        tool("get_narratives", "Declared registry tags. Mention velocity stays blank without a firehose.", id_schema()),
        tool("get_genome", "Structural genome dimensions.", id_schema()),
        tool("get_similar", "Structural similarity. Not 'will pump like X'.", id_schema()),
        tool("get_claims", "SUPPORTS / CONTRADICTS / INSUFFICIENT from live layers.", id_schema()),
        tool("get_history", "Persisted market history. Simulated excluded.", json!({
            "type": "object",
            "properties": { "id": { "type": "string" }, "range": { "type": "string", "enum": ["7d", "30d", "90d"] } },
            "required": ["id"]
        })),
        tool("get_briefing", "Daily ecosystem brief from Twin snapshots.", json!({ "type": "object", "properties": {} }))
    ])
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema,
        "annotations": { "readOnlyHint": true, "destructiveHint": false, "openWorldHint": false }
    })
}

fn id_schema() -> Value {
    json!({
        "type": "object",
        "properties": { "id": { "type": "string", "description": "ecosystem / token id, e.g. pepe" } },
        "required": ["id"]
    })
}

async fn call_tool(st: &AppState, params: &Value) -> Result<Value, String> {
    let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    if matches!(name, "execute" | "trade" | "order" | "refresh" | "onboard" | "charge") {
        return Err("EXECUTE is off. This MCP is read-only.".into());
    }
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    let id = args.get("id").and_then(|v| v.as_str()).unwrap_or("").trim();
    if name == "health" {
        return Ok(json!({
            "content": [{ "type": "text", "text": pack(crate::status_payload(st).await) }],
            "isError": false
        }));
    }
    let snaps = st.snapshots.read().await;
    let take = |id: &str| -> Result<TokenSnapshot, String> { snap(&snaps, id).cloned() };
    let text = match name {
        "list_ecosystems" => {
            let rows = crate::twin_api::ecosystems_from(&snaps);
            pack(json!({ "ecosystems": rows, "execute": false }))
        }
        "get_ecosystem" => {
            require_id(id)?;
            let s = take(id)?;
            pack(json!({ "ecosystem": ecosystem::from_snapshot(&s), "execute": false }))
        }
        "get_asset" => {
            require_id(id)?;
            pack(crate::exchange::asset_card(&take(id)?))
        }
        "get_twin" => {
            require_id(id)?;
            pack(json!({ "twin": twin::from_snapshot(id, &take(id)?) }))
        }
        "get_risk" => {
            require_id(id)?;
            pack(json!({ "risk": take(id)?.risk, "execute": false }))
        }
        "get_narratives" => {
            require_id(id)?;
            pack(json!({
                "declared": take(id)?.token.narratives,
                "mention_velocity": null,
                "note": "Mention velocity stays blank without a firehose licence."
            }))
        }
        "get_genome" => {
            require_id(id)?;
            pack(json!({ "genome": take(id)?.genome }))
        }
        "get_similar" => {
            require_id(id)?;
            let s = take(id)?;
            let others: Vec<_> = snaps.values().cloned().collect();
            pack(json!({
                "similar": twin_similarity::rank_against(&s, &others),
                "disclaimer": "Structural similarity. Not a claim that performance will repeat."
            }))
        }
        "get_claims" => {
            require_id(id)?;
            pack(json!({
                "claims": claims::from_snapshot(&take(id)?),
                "note": "INSUFFICIENT is not a quiet market. SUPPORTS is not SAFE."
            }))
        }
        "get_history" => {
            require_id(id)?;
            let _ = take(id)?;
            drop(snaps);
            let range = args.get("range").and_then(|v| v.as_str()).unwrap_or("30d");
            let days = match range {
                "7d" => 7,
                "90d" => 90,
                _ => 30,
            };
            let points = if let Some(pool) = &st.pool {
                crate::db::history(pool, id, days)
                    .await
                    .map_err(|e| e.to_string())?
            } else {
                vec![]
            };
            return Ok(json!({
                "content": [{ "type": "text", "text": pack(json!({ "token_id": id, "range_days": days, "points": points, "simulated_excluded": true })) }],
                "isError": false
            }));
        }
        "get_briefing" => {
            let rows: Vec<_> = snaps
                .values()
                .map(|s| {
                    json!({
                        "id": s.token.id,
                        "headline": s.briefing.headline,
                        "data_state": s.data_state.as_str(),
                    })
                })
                .collect();
            pack(json!({ "items": rows, "execute": false }))
        }
        "" => return Err("tool name required".into()),
        other => return Err(format!("unknown tool {other}")),
    };
    Ok(json!({
        "content": [{ "type": "text", "text": text }],
        "isError": false
    }))
}

fn require_id(id: &str) -> Result<(), String> {
    if id.trim().is_empty() {
        return Err("id is required".into());
    }
    Ok(())
}

fn snap<'a>(
    snaps: &'a std::collections::HashMap<String, TokenSnapshot>,
    id: &str,
) -> Result<&'a TokenSnapshot, String> {
    snaps.get(id).ok_or_else(|| format!("ecosystem not found: {id}"))
}

fn pack(v: Value) -> String {
    serde_json::to_string_pretty(&v).unwrap_or_else(|_| "{}".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_is_read_only() {
        let m = manifest();
        assert_eq!(m["execute"], false);
        let tools = m["tools"].as_array().unwrap();
        assert!(tools.iter().any(|t| t["name"] == "get_claims"));
        assert!(tools.iter().any(|t| t["name"] == "get_twin"));
        assert!(!tools.iter().any(|t| t["name"] == "execute"));
    }
}
