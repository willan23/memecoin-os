# MCP — MemeCoin OS

Read-only **Model Context Protocol** for Exchange Intelligence. EXECUTE is never exposed. Missing layers stay blank.

## Live

| Transport | Where |
| --- | --- |
| HTTP JSON-RPC | `GET/POST https://memecoin-os.web.app/v1/mcp` (Hosting rewrite → Cloud Run) |
| Direct API | `https://memecoin-os-api-763598651987.europe-west1.run.app/v1/mcp` |
| Stdio | `node packages/mcp/server.mjs` |
| Cursor | `.cursor/mcp.json` → server `memecoin-os` |

`GET /v1/mcp` returns the tool manifest (`execute: false`).  
`POST /v1/mcp` is JSON-RPC 2.0 (`initialize`, `ping`, `tools/list`, `tools/call`). Batch is not supported.

## Tools

All read-only. `id` is an ecosystem / token id (`pepe`, `kishu-inu`, …).

| Tool | Purpose |
| --- | --- |
| `health` | Process + provider health |
| `list_ecosystems` | Catalog |
| `get_ecosystem` | Ecosystem + asset |
| `get_asset` | Listing card (health, risk, holders) |
| `get_twin` | Digital Twin snapshot |
| `get_risk` | Multi-factor risk. UNKNOWN when evidence is missing |
| `get_narratives` | Registry tags. Mention velocity blank without a firehose |
| `get_genome` | Structural dimensions |
| `get_similar` | Structural similarity — not “will pump like X” |
| `get_claims` | SUPPORTS / CONTRADICTS / INSUFFICIENT from live layers (same derivation as REST) |
| `get_history` | Persisted market history (`7d` / `30d` / `90d`) |
| `get_briefing` | Daily headlines from Twin snapshots |

Refused names: `execute`, `trade`, `order`, `refresh`, `onboard`, `charge`.

## Example

```http
POST /v1/mcp
Content-Type: application/json

{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"get_risk","arguments":{"id":"pepe"}}}
```

Stdio forwards each stdin line to the live HTTP endpoint (`MEMECOIN_OS_API`).

## SDKs

- TypeScript: `mcpManifest()`, `mcpCall(name, args)`
- Python: `mcp_manifest()`, `mcp_call(name, **arguments)`

HTTP smoke (2026-09-06): `get_risk` on `pepe` against Hosting rewrite → Cloud Run `00024`. Cursor still needs Settings → MCP → enable `memecoin-os`.

Integrator UI: `/exchange`.
