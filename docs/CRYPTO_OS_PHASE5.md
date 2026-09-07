# Crypto Ecosystem OS — Phase 5 scale foundation

Additive. EXECUTE off. No fake payments, no fake SSO users, no invented social data.

Token intelligence stays **shared public data**. Isolation is keys, usage, queue, members and sessions.

## Delivered

| Item | Behavior |
| --- | --- |
| Time-series | BRIN on `market_snapshots.as_of` / `liquidity_snapshots.as_of`. `GET /v1/admin/timeseries?days=90` is preview only (no delete). |
| Raw object store | Content SHA-256 on disk (`OBJECT_STORE_PATH` or `data/raw`). Flag `jobs.raw_store` **off** by default. |
| Queue | Postgres `work_queue` + `SKIP LOCKED`. Worker every 5s when `jobs.queue`. Kinds `refresh_token`, `domain_event`, `research` complete as recorded work. |
| Multi-tenant | `tenants.plan` / `status`. `GET/POST /v1/tenants`. Create requires `platform_admin` (local operator is that role). |
| Billing / metering | Plans free → enterprise. Daily meters. `METERING_ENFORCE=true` returns 402 when over quota. No Stripe. `-1` = unlimited. |
| SSO | OIDC when `OIDC_ISSUER` + client id/secret are set. Login stores state; callback uses **userinfo** (not unverified JWT-only). Unset = not configured. |
| Enterprise aliases | `/v2/discovery`, `/v2/narratives`, `/v2/research`, `/v2/intelligence`, `/v2/clusters` — same handlers as v1. v1 stays compatible. |

## API

`GET /v1/billing` · `GET /v1/queue` · `GET /v1/tenants` · `POST /v1/tenants`  
`GET /v1/auth/sso` · `GET /v1/auth/oidc/login` · `GET /v1/auth/oidc/callback`  
`GET /v1/admin/storage` · `GET /v1/admin/timeseries`

RLS on `usage_meters` / `work_queue` isolates only when `app.tenant_id` is set. Unset = local operator sees all.

## Flags / env

`jobs.queue` (on) · `jobs.raw_store` (off) · `plane.billing` · `plane.sso`  
`METERING_ENFORCE` · `OIDC_*` · `OBJECT_STORE_PATH`

## Not this wave (Phase 5)

Stripe charges, IdP user provisioning without a live issuer, per-tenant token silos, Solana, social firehose, autonomous trading.

Those items (except fake SSO / trading) are scheduled in **[NEXT_WAVE.md](NEXT_WAVE.md)** (2026-09-01). Twin slice: [DIGITAL_TWIN_ROADMAP.md](DIGITAL_TWIN_ROADMAP.md).
