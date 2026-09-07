---
name: memecoin-os
description: >-
  Operate MemeCoin OS (ecosystem intelligence). Use when changing tokens,
  Twin, exchange API, MCP, holders, community, billing, or deploy.
---

# MemeCoin OS

Token-agnostic crypto ecosystem intelligence. Not a price tracker. EXECUTE stays off.

## Live

- UI: https://memecoin-os.web.app
- API: Cloud Run `memecoin-os-api` in GCP project **`memecoin-os`** (`europe-west1`)
- Never deploy to `varejo-auditor-745a9`. Always `--project=memecoin-os`.

## Hard no

- Invent social mentions, organicness, holders, transfers, or Twin motion
- Auto-VERIFIED, pay-to-rank, price forecasts, EXECUTE
- Accept RPC / Stripe / NVIDIA / Discord / Telegram keys from chat
- Fake Stripe customers or SSO users
- Timescale/Neo4j without a written benchmark

## Sources

- Market: CoinGecko + DexScreener
- Holders: Ethplorer on Ethereum (throttled). Other EVM needs `ETHERSCAN_API_KEY`. Solana needs `SOLANA_RPC_URL`
- Last-good holders carry forward on provider failure (stale, not invented)
- Firehose: only with `FEATURE_SOCIAL` + licence. First-party chat is a different plane
- Official links may come from registry, project claim, or DexScreener `info` (https only)

## MCP

- HTTP: `GET/POST /v1/mcp` (read-only JSON-RPC)
- Stdio: `node packages/mcp/server.mjs`
- Tools never execute trades or invent numbers

## Deploy

```
cd apps/web && npm run export
firebase deploy --only hosting
gcloud run deploy memecoin-os-api --source . --region europe-west1 --allow-unauthenticated --quiet --project=memecoin-os
```
