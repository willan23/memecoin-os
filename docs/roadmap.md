# Roadmap

## Shipped (this repo)

Token-agnostic intelligence: market, DEX liquidity, holder counts, honest social `MISSING`, GitHub, health/risk/genome, grounded research, compare, rankings, alerts + webhooks, onboard, Settings, REST v1, TS + Python SDKs. Seeds: AIDOGE, Baby Doge, Kishu, PEPE.

Crypto OS:

1. Domain events, pools, wallet classes, baselines (now 1h–90d + MAD, `024`)  
2. ERC-20 indexer, whale/CEX, clusters  
3. Discovery, verification, narratives, genome clusters  
4. Research agents + lexical RAG  
5. Queue, object store, tenants, metering, OIDC, `/v2` aliases  

Postgres/Redis durable; in-memory if Docker is down. EXECUTE off.

## Next — operator env

Applied 2026-09-05: [NEXT_WAVE.md](NEXT_WAVE.md). Applied 2026-09-06: claims persist, baseline windows + MAD, `/watchlist` (Cloud Run `00024`). Leftovers: [NEXT_STEPS.md](NEXT_STEPS.md). Timescale still **not yet** ([TIMESERIES_BENCHMARK.md](TIMESERIES_BENCHMARK.md)). Production LLM still blocked. Stripe charges wait for `.env` keys.

## Blocked or deferred until a key / argument exists

- Social firehose (licence + `SOCIAL_FIREHOSE_*`)
- Production LLM gateway
- Stripe charges (meters exist; connect via `.env` — see [MONETIZATION.md](MONETIZATION.md))
- Forked per-tenant Twin snapshots (watchlist isolation is the apply path; intel stays shared)
- Timescale/ClickHouse until a written benchmark
- Neo4j (not for fashion; Postgres graph first)
- White-label / Solana / WebGL — **shipped** in NEXT_WAVE (Solana holders still need RPC)

## Cost model (sketch)

Free / Pro / Research / Growth / Enterprise / API — see [MONETIZATION.md](MONETIZATION.md) and `billing.rs`. Infra: RPC, social APIs, LLM tokens, explorer keys.

## Testing

Unit: TwinState, lifecycle, graph consistency, existing scoring/risk/research eval.  
No look-ahead in reconstruction. No fake live data.
