# Crypto Ecosystem OS — Phase 1 foundation

Source prompt: `MEMECOIN OS → CRYPTO ECOSYSTEM INTELLIGENCE OS.md`. Additive only. EXECUTE stays off. No fake transfers.

## Current architecture (preserved)

Modular monolith: `tokens/*.yaml` → `crates/core` → `crates/api` → `apps/web`. REST v1, Postgres 001–014, Redis, TS + Python SDKs.

## Phase 1 delivered

| Item | How |
| --- | --- |
| Domain events | `crates/core/src/domain_events.rs` — envelope + SHA-256 fingerprint idempotency |
| EVM indexer port | `crates/core/src/indexer.rs` + existing `EvmAdapter`. Transfers **empty** until RPC cursor (Phase 2) |
| Pool discovery | DexScreener `pairAddress` → `discovered_pools` + `PoolDiscovered` |
| Wallet model | `wallet.rs` — share ≥1% = whale *candidate*, not identity |
| Historical baselines | `baseline.rs` — originally 7d/30d mean/stddev/z; anomaly if \|z\|≥2.5 on **present** series only. **Now (`024`):** also 1h/6h/24h/90d + MAD + p25/p75. |

## Not this wave (as the master prompt)

Social firehose, production LLM, RAG, billing, SSO, Solana, autonomous trading. Transfer cursor: see Phase 2.

## API (additive)

`GET /v1/tokens/{id}/baselines|pools|wallets|domain-events`  
`GET /v1/indexer`

UI: token tab **Baselines**. Flags: `jobs.baselines`, `jobs.pool_discovery`, `jobs.indexer` (off).

## Files created

`domain_events.rs`, `baseline.rs`, `wallet.rs`, `indexer.rs`, `migrations/014_intelligence_foundation.sql`, this doc.

## Files not rewritten

Scoring, risk, REST v1 shapes, token YAML, AI policy, EXECUTE path.
