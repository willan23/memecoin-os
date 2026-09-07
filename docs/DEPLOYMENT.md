# Deployment

1. Copy `.env.example` → `.env`
2. `docker compose up -d` (Postgres **5435**, Redis **6381**) — optional but required for history, webhooks, flags persistence, API keys
3. `cargo run -p memecoin-os-api` from repo root (`TOKEN_REGISTRY_PATH=tokens`)
4. `cd apps/web && npm run dev`

Useful env (see `.env.example`):

| Variable | Role |
| --- | --- |
| `DATA_MODE` | `live` (default) or `simulation` |
| `DATABASE_URL` / `REDIS_URL` | Persistence / cache |
| `FEATURE_*` | Provider/job toggles if Postgres flags are empty |
| `FEATURE_HOLDERS` | Ethplorer / Etherscan holder fetch |
| `ETHERSCAN_API_KEY` | Holder count on non-Ethereum EVM |
| `ETHPLORER_API_KEY` | Default `freekey` |
| `AUTH_REQUIRED` | Require Bearer `mcos_…` on `/v1/*` and `/v2/*` |
| `METERING_ENFORCE` | 402 when over plan quota (default off; usage still recorded) |
| `OIDC_ISSUER` / `OIDC_CLIENT_ID` / `OIDC_CLIENT_SECRET` | SSO; all required or login stays unconfigured |
| `OBJECT_STORE_PATH` | Raw snapshot objects (`jobs.raw_store` off by default) |
| `TOKEN_REFRESH_PER_HOUR` | Per-token refresh quota (default 12) |
| `NEXT_PUBLIC_API_URL` | Next rewrite target (default `http://127.0.0.1:8080`) |
| Firebase Hosting | Project `memecoin-os` — [FIREBASE.md](FIREBASE.md). Public UI: `https://memecoin-os.web.app`. API via Hosting rewrite → Cloud Run `memecoin-os-api`. Webhook: `https://memecoin-os.web.app/v1/billing/stripe/webhook` |

Backup: `scripts/backup-postgres.ps1`  
Restore: `scripts/restore-postgres.ps1 -Dump backups\....sql`

Incident: if `/health.degraded` is true, check compose health, then `DATABASE_URL` / `REDIS_URL`. The API stays up in in-memory degraded mode; history, webhooks, keys and flag writes require Postgres.

Migrations apply on API boot (`001`–`024`, including project claims, community chat, `evidence_claims`, baseline MAD columns). Restart the API after pulling new SQL.

Public publish: [FIREBASE.md](FIREBASE.md). Hosting is the UI; Cloud Run is the API (revision **`memecoin-os-api-00024-v8w`**, 2026-09-06); Cloud SQL is Postgres. `AUTH_REQUIRED` stays false so anyone can read. EXECUTE stays off. Social firehose stays MISSING without a licence. Redis on Cloud Run is still unset (rate limit / refresh lease skipped).
