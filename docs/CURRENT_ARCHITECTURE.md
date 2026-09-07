# Current architecture

Source of truth for the running system. Do not rewrite working modules.

## Shape

Token-agnostic **modular monolith**. Rust (`crates/core`, `crates/api` Axum) + Next.js (`apps/web`) + YAML registry (`tokens/`) + Postgres/Redis (compose **5435 / 6381**). In-memory fallback if Docker is down.

```
tokens/*.yaml
      ↓
IntelligenceEngine (core)
      ↓
TokenSnapshot + scores + risk + genome
      ↓
Ecosystem + TwinState + graph projection
      ↓
Postgres snapshots / twin / jobs / keys  |  Redis cache · rate · leases
      ↓
Axum /v1 + /v2 Twin  →  Next rewrite  →  terminal UI
      ↓
SSE /v2/events (API origin, not buffered by Next)
```

EXECUTE is off. No private keys. No ticker branches in `crates/core`.

## Map

| Area | Path | Notes |
| --- | --- | --- |
| Domain / providers / intelligence | `crates/core/src/` | Snapshot intelligence + Twin modules (`ecosystem`, `twin`, `twin_graph`, `twin_sim`, `twin_similarity`, `twin_replay`, `whale_behaviour`). |
| REST / jobs / auth / queue / SSO | `crates/api/src/` | Additive `/v1`. Twin `/v2`. Exchange `/v1/ecosystems*` + `/v1/mcp`. Migrations through `024`. |
| UI | `apps/web/src/app/` | Twin, briefing, simulations, projects, community, exchange, watchlist, plus token OS pages (Claims / Baselines tabs). |
| SDKs | `packages/sdk`, `packages/sdk-python`, `packages/mcp` | v1 + Twin + exchange + MCP. |
| Registry | `tokens/*.yaml` | AIDOGE, Baby Doge, Kishu, PEPE. Runtime onboard adds more (UNVERIFIED). |
| SQL | `migrations/001`–`024` | Twin + project claims + community connect + `evidence_claims` + baseline MAD/percentiles. |
| Scoring weights | `scoring/*.yaml` | Health + genome + risk versions. |

## Hexagonal rules (keep)

Product code depends on canonical models, not CoinGecko / a ticker / an RPC URL from a prompt.  
`ChainAdapter` + `EvmAdapter`. `MarketProvider` / Dex / holders / GitHub. Social firehose is a port with no live licence.

## Storage today

| Store | Role |
| --- | --- |
| PostgreSQL | Snapshots, scores, events, alerts, flags, keys, research reports, indexer cursors, tenants, usage, work_queue, object_blobs, users/sessions, Twin relationships, `evidence_claims`, `metric_baselines` |
| Redis | Cache, IP rate limits, job leases — never source of truth |
| Local FS | Content-hashed raw objects when `jobs.raw_store` (default **off**) |
| In-memory | Degraded mode |
| Broadcast bus | In-process SSE `twin.updated` |

Timescale/ClickHouse, Neo4j, vector DB: **not introduced**. BRIN on `market_snapshots.as_of` / `liquidity_snapshots.as_of`. Next: [NEXT_STEPS.md](NEXT_STEPS.md).

## Jobs

Snapshot refresh (~180s, Redis lease). Persist Twin after snapshot. On-chain indexer when RPC/Etherscan. Ecosystem discovery/verification/narratives/genome clusters. Queue worker (5s, `SKIP LOCKED`) for `refresh_token` / `domain_event` / `research` (recorded work, no invented side effects).

## What this is not

Not a price forecast engine. 3D is a projection of the Twin graph, not a second model. Token intel is shared across tenants (keys/usage/queue isolated). Simulation clones metrics, never price.
