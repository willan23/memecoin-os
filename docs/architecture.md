# Architecture

MemeCoin OS is a **token-agnostic modular monolith**.

```
DATA PLATFORM          AI PLATFORM           APPLICATIONS
On-chain / market      LLM gateway           Dashboards
Social / dev           RAG / agents          Community
News / DEX / CEX       Forecasting           API / SDK
        \                  |                     /
                 INTELLIGENCE CORE
              risk · growth · research
                       |
              TOKEN DEFINITION LAYER
              AIDOGE · BABYDOGE · KISHU · PEPE · future
```

## Boundaries

| Crate / package | Responsibility |
| --- | --- |
| `tokens/*.yaml` | Token Definition Layer. Adding a token = config, not a core PR. |
| `crates/core` | Canonical models, registry, chain trait, providers, quality, scoring, risk, growth, AI policy |
| `crates/api` | Axum REST, CORS, rate/body limits, health, jobs, auth, webhooks |
| `apps/web` | Terminal UI (`/settings`, `/watchlist`, token Claims/Baselines/Genome/Timeline, Price columns) |
| `plugins/` | Capability-declared adapters (reserved) |
| `packages/sdk` | TypeScript client |
| `packages/sdk-python` | Python client |
| `packages/mcp` | Stdio MCP (forwards to `POST /v1/mcp`) |

## Hexagonal rules

- Product modules depend on **canonical models**, never on CoinGecko, a specific RPC, or a ticker.
- `ChainAdapter` is blockchain-agnostic (`EvmAdapter` today; Solana/Sui/Aptos later).
- `MarketProvider` / `LiquidityProvider` / `OnchainProvider` / `SocialProvider` / `DevProvider` are swappable.
- Provider failure is a degraded snapshot (`validation_status: stale|missing|conflict`), not a process crash.

## Storage

| Store | Use |
| --- | --- |
| PostgreSQL | Tokens, snapshots, scores, risk, events, alerts, webhooks, flags, API keys, research, indexer, tenants, usage, work_queue, users/sessions |
| Redis | Cache, rate-limit, job leases — never source of truth |
| Local FS | Content-hashed raw objects when `jobs.raw_store` (default off) |
| In-memory | Fallback only when DB is down or `DATA_MODE=simulation` |
| ClickHouse / Timescale | Only after a Twin-history benchmark — [NEXT_WAVE.md](NEXT_WAVE.md) |
| Vector DB | Not required; research RAG is lexical over snapshots |
| Graph layer | Transfer clusters now; temporal relational graph is V2 Phase 3 |

The API persists to **Postgres** when `DATABASE_URL` is reachable (`migrations/001`–`023`); otherwise it degrades to in-memory. History, webhooks, keys, meters, chat and flag writes need Postgres.

Digital Twin: [DIGITAL_TWIN.md](DIGITAL_TWIN.md). Next apply: [NEXT_WAVE.md](NEXT_WAVE.md). Do not put Twin logic in the 3D renderer.

## Events

`TokenDiscovered`, `TokenVerified`, `LiquidityUpdated`, `HolderSnapshotCreated`, `WhaleMovementDetected`, `SocialSpikeDetected`, `RiskChanged`, `ScoreUpdated`, `RecommendationGenerated`, plus fingerprinted domain events (`014`).

Jobs persist score/liquidity/holder events when those sources are present. `work_queue` (`018`) records `refresh_token` / `domain_event` / `research`. Twin events (`EcosystemCreated`, `TwinStateUpdated`, …) land with V2 Phase 1+.

## Execution policy

```
READ = on
ANALYZE = on
RECOMMEND = on
PROPOSE = on
EXECUTE = off
```

No private keys. No autonomous market actions.

## Deployment

Local: `docker compose up -d` (optional) + `cargo run -p memecoin-os-api` + `next dev`. Kubernetes only when operationally justified.

Observability: structured tracing (`RUST_LOG`), `/health`, data freshness and provider confidence on every observation.
