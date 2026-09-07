# Current data model

## Canonical types (`crates/core/src/models.rs`)

Documented names: Token, Chain, Contract, Pool, Exchange, Wallet, Holder, Transaction, Transfer, Swap, LiquidityEvent, SocialPost, SocialMention, DeveloperEvent, NewsEvent, GovernanceProposal, TreasuryMovement, AIObservation, Signal, Alert, Recommendation.

Live centre of gravity is still **`TokenSnapshot`**: market, liquidity, on-chain, social, development, scores, risk, genome, growth, intelligence, timeline, briefing. Every external fact carries provenance (`source`, `provider`, `freshness`, `confidence`, `validation_status`, `raw_reference`) plus roll-up `data_state`: live / recent / stale / missing / conflict / simulated.

**Ecosystem** is composed from a snapshot (one candidate per YAML token). **TwinState** is the observed field map (market, liquidity, holders, wallets, whales, development, social, narrative, risk, governance, network). Reconstruct uses history points ≤ timestamp and skips `simulated`.

## Persistence (migrations 001–024)

Hot tables: `tokens`, `market_snapshots`, `liquidity_snapshots`, scores/risk, `events`, `alerts`, `briefings`, holder snapshots, `social_posts` (schema; live = MISSING), `feature_flags`, `api_keys`, `tenants`, `usage_meters`, `work_queue`, `object_blobs`, `users`, `sessions`, `oidc_states`, research reports, indexer cursors, discovered pairs, narratives, genome clusters, wallet classifications, whale events, domain events, Twin entity relationships / simulations (`019`), project claims/alerts (`021`/`022`), community connect (`023`), `evidence_claims` latest row per `(token_id, claim_id)` (`024`).

JSONB used for payloads (events, reports, queue). Not a document store.

## Intelligence overlays

| Module | What it stores / computes | Limit |
| --- | --- | --- |
| `domain_events` | Fingerprinted, idempotent | Token-scoped |
| `baseline` | 1h/6h/24h/7d/30d/90d mean/std/z + MAD + p25/p75; anomaly if \|z\|≥2.5 | Short windows MISSING until enough live points |
| `wallet` | ≥1% share = whale *candidate* | Not identity |
| `indexer` / `whale` / `cex` / `cluster` | ERC-20 logs, CEX list incomplete, hubs skipped | — |
| `whale_behaviour` | Accumulation / distribution / rotation / dormant | Empty = UNKNOWN |
| `discovery` / `verification` / `narrative` | DEX counterparts; never auto-VERIFIED; social velocity MISSING | Firehose blocked |
| `genome_cluster` / `twin_similarity` | Cosine + Jaccard + chain | Name unused |
| `ecosystem` / `twin` / `twin_graph` / `twin_replay` | Candidate Twin, reconstruct, 2D/3D projection, observed frames | Neo4j not default — [NEXT_WAVE.md](NEXT_WAVE.md) |
| `twin_sim` | Metric shocks, sensitivity | Not a price forecast |
| `evidence` / `claims` / `research` / `agents` | Lexical RAG + derived SUPPORTS/CONTRADICTS/INSUFFICIENT; latest row in `evidence_claims` | Not bitemporal. No invented evidence |
| `billing` / `storage` | Plans + content-hash objects | Isolation ≠ per-tenant tokens |

## Provenance rule (do not break)

Never invent a fact to fill MISSING. Simulation rows must stay labelled. EXECUTE stays off.
