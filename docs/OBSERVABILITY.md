# Observability

`GET /health` and `GET /status` (same JSON). UI page `/status` calls `/health` so Next does not steal the route.

Fields:

- `degraded` — true when Postgres is down or `DATA_MODE=simulation`
- `postgres` / `redis` — ok + latency_ms
- `tokens` — live / missing / conflict / stale counts
- `twin.sse` — `/v2/events`
- `last_job` — snapshot_refresh lag_ms
- `providers` — rows from `provider_health` when present (UI table, not raw dump)
- `policy.execute` — always false

Logs: `tracing-subscriber` (`RUST_LOG=info,memecoin_os_api=debug,memecoin_os_core=debug`).
Job lease: Redis `lock:snapshot_refresh`.
Manual refresh: `POST /v1/tokens/{id}/refresh` and `POST /v1/jobs/refresh` (Settings).

## Incident runbook

| Symptom | Likely cause | Action |
| --- | --- | --- |
| CoinGecko 429 / mcap $0 | Public rate limit | Wait; quota is 12 refreshes/token/hour. DexScreener may still fill **spot price**. Do not invent mcap. |
| Price column empty / `—` | Market `MISSING` (no CG and no DEX price) | Check `/health` providers and `FEATURE_COINGECKO` / `FEATURE_DEXSCREENER`. |
| DexScreener down / liquidity MISSING | Provider outage | UI shows MISSING. Job records `provider_health.dexscreener`. |
| Holders MISSING on ETH | Ethplorer 429 or down | Retry later. Non-ETH needs `ETHERSCAN_API_KEY`. |
| `/health.degraded=true`, postgres.ok=false | Docker/DB down | Start Docker Desktop, `docker compose up -d` (ports 5435 / 6381). API stays up in-memory. |
| Job `lag_ms` climbing, tokens_err > 0 | Provider timeouts or lease contention | Check Redis `lock:snapshot_refresh` and last `job_runs.error`. |
| Webhooks / keys / flag writes fail | Postgres down | Hooks and keys require Postgres. Confirm `/v1/flags` env fallback vs table. |
| 401 on `/v1/*` | `AUTH_REQUIRED=true` without Bearer | Create a key in Settings or set `AUTH_REQUIRED=false`. |
