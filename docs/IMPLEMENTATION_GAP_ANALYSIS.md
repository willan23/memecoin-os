# Implementation gap analysis

Columns: Implemented / Partially / Mocked / Missing / Broken / Needs Refactor / Blocked (external key).

| Area | State | Notes |
| --- | --- | --- |
| Token YAML registry | Implemented | PEPE + AIDOGE + Baby Doge + Kishu. No ticker branches in `crates/core`. |
| Live CoinGecko market | Implemented | `DATA_MODE=live` default. 429 → mcap/volume 0; DexScreener may still supply spot price. |
| DexScreener liquidity + secondary price | Implemented | Relative divergence > 25% → `DATA_CONFLICT`. |
| Overview `price_usd` | Implemented | Cards + CSV + UI Price column. Tiny prices not scientific notation. |
| DataState LIVE/RECENT/STALE/MISSING/CONFLICT/SIMULATED | Implemented | Fixtures only when `DATA_MODE=simulation`. |
| GitHub DevProvider | Implemented | Optional `socials.github`. |
| Scoring skip + renormalize | Implemented | `scoring/ecosystem-health-1.0.0.yaml`. |
| Risk UNKNOWN on missing evidence | Implemented | Social UNKNOWN in live mode. Concentration UNKNOWN until top-holder sample exists. |
| Onboard EVM + YAML persist | Implemented | 20-byte `0x` + allow-listed chains. |
| Postgres snapshots/scores/risk/events/alerts | Implemented | SQLx + migrations 001–013. In-memory if DB down or simulation. |
| Redis cache / rate-limit / job leases | Implemented | API 180/min, AI 20/min, refresh quota 12/hour. |
| Historical series 7d/30d/90d + CSV | Implemented | Simulated rows excluded. Empty without Postgres. |
| Alert engine + Discord/Telegram webhooks | Implemented | Fingerprint dedupe + per-kind rules + ack. |
| Daily briefing 3/2/3 + evidence | Implemented | Never treats holder/social zeros as changes. |
| Prompt injection defense | Implemented | Grounded-rules refuse override / EXECUTE. |
| Status page /health | Implemented | UI `/status` fetches `/health`. |
| UI filters + Settings | Implemented | Compare, rankings, ecosystems. Genome, Timeline, `/settings`. |
| Feature flags | Implemented | Table + PATCH + Settings + `FEATURE_*` env fallback. |
| API keys | Implemented | Optional. `AUTH_REQUIRED=false` by default. SHA-256 stored. |
| Holder counts | Implemented | Ethplorer on Ethereum. Etherscan v2 when `ETHERSCAN_API_KEY` is set. |
| Domain events / pools / baselines | Implemented | 1h/6h/24h/7d/30d/90d + MAD/p25/p75 (`024`). Anomalies only on live series with enough points. |
| Wallet classification | Implemented | Share-based whale/retail/unknown. Not identity. |
| Transfer / whale movement indexer | Implemented | RPC cursor + Etherscan fallback. Empty without keys. |
| CEX flow / wallet clusters | Implemented | Public CEX list (incomplete). Hubs excluded from clusters. |
| Discovery / verification / narratives | Implemented | Dex counterparts only. Never auto-VERIFIED. Social velocity MISSING. |
| Genome clustering | Implemented | Genome + narrative + chain. Name unused. |
| AI research / RAG / reports | Implemented | Lexical RAG + specialists. No invented evidence. No live LLM. |
| Social firehose | Missing | Schema only. Live mode = MISSING. **Blocked** until licence. Path: [NEXT_WAVE.md](NEXT_WAVE.md). |
| Production LLM | Missing | Grounded template agent only. **Blocked** (gateway + key). |
| Multi-tenant billing / SSO | Implemented | Phase 5. Metering without Stripe. OIDC when issuer is set. Token data stays shared. Stripe = `.env` later. |
| White-label / watchlist | Implemented | Branding JSON + `/watchlist` + client filters. Server isolation when `AUTH_REQUIRED`. Not forked Twins. |
| Ecosystem entity / TwinState | Implemented | One candidate ecosystem per YAML token. Composed from snapshots. |
| Historical Twin reconstruct | Implemented | `?ago=` / `?at=`; nearest observation ≤ T; simulated excluded. |
| Knowledge graph | Partial | 2D relational projection + `entity_relationships`. Temporal Postgres edges next; Neo4j not default. |
| Evidence / Claim graph | Partial | Derived SUPPORTS/CONTRADICTS/INSUFFICIENT + latest row in `evidence_claims`. Not bitemporal. |
| Lifecycle / similarity 2.0 | Implemented | Evidence-only phases. Genome + chain + narratives. Not price. |
| Simulation / what-if | Implemented | Health/risk shocks. Missing fields do not move. Not a prediction. |
| Twin UI / copilot | Implemented | `/twin`, isometric 3D, replay, SSE, `/simulations`, `POST /v2/copilot`. Motion only from real events. |
| Solana | Implemented (adapter) | Live RPC only if `SOLANA_RPC_URL`. Holders/transfers blank without indexer. |
| Exchange API / MCP / chat / project desk | Implemented | See [EXCHANGE.md](EXCHANGE.md), [MCP.md](MCP.md), [COMMUNITY.md](COMMUNITY.md), [PROJECTS.md](PROJECTS.md). |
