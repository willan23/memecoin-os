# Próximos passos

Updated 2026-09-06 after claims persist + baseline windows + watchlist polish. Production: Hosting `memecoin-os` · Cloud Run **`memecoin-os-api-00024-v8w`** in GCP project **`memecoin-os`**. Do not deploy to `varejo-auditor-745a9`.

The product is usable. What follows needs **operator env or a licence** — do not invent the numbers in the meantime.

## Shipped in this wave (code)

| Item | What landed |
| --- | --- |
| Persist SUPPORTS/CONTRADICTS | `evidence_claims` (migration `024`). Latest row per `(token_id, claim_id)` on refresh when `jobs.claims_persist`. GET `/v1/ecosystems/{id}/claims` still derives live; `persisted` is last stored. Token page Claims tab. |
| Baseline 1h / 6h / 24h / 90d + MAD | Job slices 90d history. MAD + p25/p75 stored additively. 1h/6h stay MISSING until enough live points exist. |
| Watchlist UI | Symbols/names, `/watchlist`, Overview + Ecosystems client filter, Settings save confirmation. Isolation still only when `AUTH_REQUIRED`. |
| Official-link refresh | Already on Dex pull — refresh tokens after deploy. |

## Do first (when you sit down)

1. **Enable Cursor MCP** — HTTP `get_risk` on `pepe` already passed (2026-09-06). Remaining: Settings → MCP → enable `memecoin-os` (`.cursor/mcp.json`).
2. **Etherscan** — key is on Cloud Run (`memecoin-os-etherscan-key` → `ETHERSCAN_API_KEY`, revision `00025`). `tokenholdercount` is a **Pro** endpoint; free keys get NOTOK (BSC also: free plan has no chain coverage). Holders on BABYDOGE / BRETT / AIDOGE stay blank until the Etherscan plan covers those chains. Never paste the key into chat. Local `.env` name must be exactly `ETHERSCAN_API_KEY`.
3. **Solana RPC** — `SOLANA_RPC_URL` on Cloud Run if you want a real Solana adapter path. Holders/transfers stay blank until an indexer exists.
4. **Discord / Telegram** — create the app/bot yourself; set `DISCORD_CLIENT_ID`, `DISCORD_CLIENT_SECRET`, `TELEGRAM_BOT_TOKEN`, `TELEGRAM_BOT_USERNAME`, `TELEGRAM_WEBHOOK_SECRET` on Cloud Run. Redirect: `https://memecoin-os.web.app/v1/connect/discord/callback`.
5. **Stripe Checkout** — secret + publishable + webhook secret are in local `.env`. Live Prices created (EUR/mo): Pro 199 · Research 299 · Growth 399 · API 499 · Enterprise 999 (`STRIPE_PRICE_*`). Still need: Stripe Dashboard webhook → `https://memecoin-os.web.app/v1/billing/stripe/webhook` (events: `checkout.session.completed`, `customer.subscription.updated`, `customer.subscription.deleted`) and sync keys + price ids to Cloud Run. Settings must not show a Stripe account id.
6. **Google login** — code ready (`/login`, OIDC discovery). Create OAuth Web client in Google Cloud; put `OIDC_ISSUER=https://accounts.google.com` + client id/secret in `.env` / Cloud Run. Redirect: `https://memecoin-os.web.app/v1/auth/oidc/callback`.

## Still product (honest, no new vendors)

| Item | Why | Gate |
| --- | --- | --- |
| Redis on Cloud Run | Rate limit / refresh lease skipped when unset | Memorystore or skip |

## Blocked (do not start)

| Item | Why |
| --- | --- |
| Social mention firehose | Licence + `SOCIAL_FIREHOSE_*` |
| Production LLM polish | NVIDIA/gateway already local-only; never accept a key in chat |
| Timescale | Write [TIMESERIES_BENCHMARK.md](TIMESERIES_BENCHMARK.md) numbers first |
| Neo4j | Temporal Postgres first |
| Solana holder counts without RPC/indexer | Do not scrape a random public API into LIVE |
| Auto-VERIFIED / pay-to-rank | Never |

## Deploy reminder

```
cd apps/web && npm run export
firebase deploy --only hosting
gcloud run deploy memecoin-os-api --source . --region europe-west1 --allow-unauthenticated --quiet --project=memecoin-os
```

Always `--project=memecoin-os`.

## Hard no (unchanged)

Invented social/holders/transfers · price forecasts · EXECUTE · keys from prompts · fake Stripe/SSO · decorative Twin motion · rewrite of working Twin/core.
