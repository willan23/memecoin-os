# API

REST v1 is additive. New fields (`data_state`, `price_usd` on cards, `degraded`, `fingerprint`) must not break existing clients.

Base: `http://127.0.0.1:8080`. The Next.js app proxies `/v1/*`, `/v2/*` and `/health`.

## Routes

| Method | Path | Notes |
| --- | --- | --- |
| GET | `/health` `/status` | Same JSON. UI `/status` calls `/health`. |
| GET | `/v1/overview` | Token cards include `price_usd`, `market_state`, `holders_state`. |
| GET | `/v1/overview.csv` | Includes `price_usd`. |
| GET/POST | `/v1/tokens` | POST = onboard EVM wizard. |
| GET | `/v1/tokens/{id}` | Full snapshot. |
| POST | `/v1/tokens/{id}/refresh` | One token; Redis quota `TOKEN_REFRESH_PER_HOUR` (default 12). |
| GET | `/v1/tokens/{id}/{market,onchain,social,development,risk,growth,intelligence,timeline,briefing,genome}` | Part routes. |
| GET | `/v1/tokens/{id}/history` `history.csv` | `?range=7d\|30d\|90d`. Simulated rows excluded. Empty if Postgres down. |
| GET | `/v1/tokens/{id}/baselines` `pools` `wallets` `domain-events` | Windows 1h/6h/24h/7d/30d/90d. Fields include `mad`, `percentile_25`, `percentile_75` (serde default 0/null on old rows). Wallets are classes, not identity. |
| GET | `/v1/tokens/{id}/transfers` `whale-events` `clusters` | Indexed ERC-20 logs only. Empty = MISSING, not zero activity. |
| GET | `/v1/indexer` | RPC / Etherscan flags + persisted cursors. |
| GET | `/v1/discovery` `/v1/narratives` `/v1/genome-clusters` | Phase 3. Discovery never auto-verifies. Social velocity MISSING. |
| GET | `/v1/tokens/{id}/verification` | Computed level + declared YAML. VERIFIED ≠ SAFE. |
| GET | `/v1/tokens/{id}/briefing.md` | Markdown export. |
| GET | `/v1/compare?ids=` | Comma-separated token ids. |
| GET | `/v1/rankings` | Health, momentum, risk, development, community, liquidity. |
| GET | `/v1/alerts` | Unacknowledged only when Postgres is up. |
| PATCH | `/v1/alerts/rules` | Body `{ "kind", "enabled" }`. |
| POST | `/v1/alerts/{id}/ack` | Requires Postgres. |
| GET/POST | `/v1/webhooks` | Discord or Telegram. HTTPS (or `http://127.0.0.1` for tests). |
| DELETE | `/v1/webhooks/{id}` | |
| GET | `/v1/flags` | Postgres rows, else `FEATURE_*` env fallback. |
| PATCH | `/v1/flags/{key}` | Body `{ "enabled", "reason?" }`. |
| GET/POST | `/v1/keys` | POST returns `secret` once. |
| DELETE | `/v1/keys/{id}` | Revoke. |
| POST | `/v1/jobs/refresh` | Full snapshot job. |
| POST | `/v1/ai/query` | Body `{ "question", "token_id?" }`. Grounded research markdown. |
| POST | `/v1/research` | Full report JSON (sections, unknown, limitations). |
| GET | `/v1/tokens/{id}/research.md` `.json` | Latest persisted report or on-the-fly grounded report. |
| GET | `/v2/ecosystems` | Candidate Twin per registry token. `auto_verified` always false. |
| GET | `/v2/ecosystems/compare?ids=` | Twin A vs B + structural similarity. |
| GET | `/v2/ecosystems/{id}/twin/replay` | Observed frames only (`?range=24h\|7d\|30d`). |
| GET | `/v2/ecosystems/{id}/whales` | Whale behaviour aggregate. Empty = UNKNOWN. |
| GET | `/v2/briefing` | Daily ecosystem headlines from Twin snapshots. |
| GET | `/v2/events` | SSE `twin.updated` + keepalive. |
| GET | `/v2/ecosystems/{id}` | Ecosystem + current Twin. |
| GET | `/v2/ecosystems/{id}/twin` `/twin/state` `/twin/history` `/twin/graph` `/twin/evolution` | State, reconstruct `?ago=24h\|7d\|30d` or `?at=RFC3339` (no look-ahead), 2D graph, 24h delta. |
| GET | `/v2/ecosystems/{id}/genome` `/lifecycle` `/anomalies` `/evidence` `/similar` | Fingerprint, lifecycle, observed sequence, claims, structural similarity. |
| POST | `/v2/simulations` | What-if on Twin metrics. Banner: not a prediction. Price not simulated. |
| GET | `/v2/simulations/{id}` | Persisted result (Postgres). |
| POST | `/v2/copilot` | Research + Investigate Next from Twin. EXECUTE off. |
| GET/POST | `/v1/tenants` | POST requires `platform_admin`. Token data stays shared. |
| GET | `/v1/branding` | Current tenant chrome. |
| GET/PATCH | `/v1/tenants/{id}/branding` | HTTPS logo only. |
| GET/PUT | `/v1/watchlist` | Token ids. Falls back to tenant `default`. Server filters lists when `AUTH_REQUIRED`. UI `/watchlist` + client filters always. |
| GET | `/v1/billing` | Plan, daily usage, limits, `stripe.configured`. 402 only if `METERING_ENFORCE=true`. |
| POST | `/v1/billing/stripe/webhook` | Signature required. 503 if secret unset. |
| GET | `/v1/queue` | Pending / locked work. |
| GET | `/v1/admin/storage` | Object-store path + blob count. |
| GET | `/v1/admin/timeseries` | `?days=90` prune **preview** (no delete). |
| GET | `/v1/auth/sso` | `{ configured, issuer? }`. |
| GET | `/v1/auth/oidc/login` | 302 to IdP, or 503 if unset. |
| GET | `/v1/auth/oidc/callback` | Code exchange + userinfo. Session returned once. |
| GET/POST | `/v2/discovery` `/v2/narratives` `/v2/clusters` `/v2/research` `/v2/intelligence` | Aliases of v1. Additive. |

## Overview card

Each `tokens[]` row includes `price_usd` (spot). Market cap / volume stay 0 when CoinGecko is down even if DexScreener filled `price_usd`. UI formats tiny prices without scientific notation.

## Auth

Optional. `AUTH_REQUIRED=false` by default (local operator).  
`AUTH_REQUIRED=true` requires `Authorization: Bearer mcos_…` on `/v1/*` and `/v2/*` (`/health`, `/status`, `/v1/auth/*` stay open). Secrets are SHA-256 hashed; only a prefix is stored for lookup.

Rate limits (Redis): API 180/min, AI 20/min per IP. Without Redis, limits are skipped.

Daily plan meters increment after auth. Enforcement is off unless `METERING_ENFORCE=true`.

## SDKs

- TypeScript: `packages/sdk` (`@memecoin-os/sdk`)
- Python: `packages/sdk-python` (`MemeCoinOsClient`)
- MCP: [MCP.md](MCP.md) — `GET/POST /v1/mcp`, stdio `packages/mcp`

## Exchange / projects / community (additive)

| Method | Path | Notes |
| --- | --- | --- |
| GET | `/v1/health` | Exchange envelope around process health |
| GET | `/v1/ecosystems` `/{id}` `/asset` `/twin` `/risk` `/narratives` `/genome` `/similar` `/history` | [EXCHANGE.md](EXCHANGE.md) |
| GET | `/v1/ecosystems/{id}/claims` | Derived live `claims` + last stored `persisted`. SUPPORTS / CONTRADICTS / INSUFFICIENT |
| GET/POST | `/v1/mcp` | Manifest + JSON-RPC. Read-only |
| GET | `/v1/projects` `/{id}` `/alerts` | [PROJECTS.md](PROJECTS.md) |
| POST | `/v1/projects/{id}/claim` `/unclaim` | Operator key when configured |
| PATCH | `/v1/projects/{id}/profile` | https official links |
| GET | `/v1/community` `/rooms` `/rooms/{id}/messages` | [COMMUNITY.md](COMMUNITY.md) |
| POST | `/v1/community/session` `/rooms/{id}/messages` | First-party chat LIVE |
| GET | `/v1/connect` `/discord` `/discord/callback` `/telegram` | OAuth port; unset = not configured |
| POST | `/v1/connect/telegram/inbound` | Bot webhook |
