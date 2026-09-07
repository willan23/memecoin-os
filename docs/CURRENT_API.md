# Current API

Canonical route table: [API.md](API.md). This file is the Twin-wave map.

## Compatibility

`/v1/*` is **additive-only**. Clients must keep working. `/health` and `/status` stay public.

`/v2/discovery` `/v2/narratives` `/v2/research` `/v2/intelligence` `/v2/clusters` are **aliases of v1 handlers**. Twin resources live under `/v2/ecosystems`, `/v2/simulations`, `/v2/copilot`, `/v2/briefing`, `/v2/events`.

## Present (do not duplicate)

Token snapshot + parts, history/CSV, compare, rankings, alerts/ack/rules, webhooks, flags, keys, onboard, refresh, baselines/pools/wallets/domain-events, transfers/whale-events/clusters, indexer, discovery/narratives/genome-clusters/verification, research + briefing, AI query, tenants, billing, queue, admin storage/timeseries, OIDC login/callback.

Exchange Intelligence: `/v1/ecosystems*` + `/v1/health` envelope, `/v1/ecosystems/{id}/claims` (derived + `persisted`), `GET/POST /v1/mcp`.

Watchlist: `GET/PUT /v1/watchlist` — default tenant when no `TenantCtx`. Server list isolation only when `AUTH_REQUIRED`.

Projects: `/v1/projects*` (claim is a label, not VERIFIED).

Community: `/v1/community*` chat LIVE; `/v1/connect*` Discord/Telegram port.

Twin: ecosystems list/get, twin now/state/history/graph/evolution/replay, genome, lifecycle, anomalies, evidence, similar, whales, compare, simulations, copilot, daily briefing, SSE.

## Auth / meter

`AUTH_REQUIRED=false` default. Bearer `mcos_…` when on. Local operator = tenant `default`, role `platform_admin`.  
Meters increment after auth. `402` only if `METERING_ENFORCE=true`.  
OIDC only when `OIDC_ISSUER` + client id/secret are set. Identity from **userinfo**.  
`GET /v2/events` is open like `/health` (keepalive + `twin.updated`).

## SDKs

TypeScript `packages/sdk`, Python `packages/sdk-python`, MCP `packages/mcp`. Twin methods: `ecosystems`, `twin`, `twinReplay`, `whaleBehaviour` / `whale_behaviour`, `compareTwins` / `compare_twins`, `briefing`, `simulate`, `copilot`. Exchange: `exchange*`, `mcpManifest` / `mcpCall`. Keep v1 methods.
