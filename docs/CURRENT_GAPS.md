# Current gaps (product target)

Honest map. “Partially” means reuse — do not rebuild from zero.

| V2 item | State | Reuse |
| --- | --- | --- |
| Ecosystem entity | Done | One candidate ecosystem per YAML token. `auto_verified` always false. |
| TwinState / temporal reconstruct | Done | `GET …/twin/state?ago=` / `?at=` — nearest observation ≤ T; simulated skipped. |
| Time Machine / replay / compare A vs B | Done | Replay frames + `/v2/ecosystems/compare` + UI `/twin/compare`. |
| Knowledge graph (temporal edges) | Partial | 2D/3D relational projection persisted. Not Neo4j / full bitemporal. |
| Evidence / Claim engine | Partial | Derived live + latest row in `evidence_claims` (`024`). GET still derives; `persisted` is last stored. Not bitemporal / CORROBORATES / INVALIDATES. |
| Ecosystem Genome 2.0 + DNA fingerprint | Partial | Token genome + clusters + Twin genome route. |
| Similarity engine (structural) | Done | Genome + Jaccard + chain. Disclaimer is not “will pump like X”. |
| Lifecycle state machine | Done | Evidence-only phases. Missing social ≠ GROWTH. |
| Emerging ecosystem discovery | Partial | DEX counterpart discovery. Never auto-VERIFIED. |
| Anomaly *propagation* | Partial | Baseline z-score per metric; Twin anomalies route. |
| Narrative engine (open clustering) | Partial | Tags + name heuristics; firehose MISSING. |
| Wallet cluster 2.0 (funding/timing) | Partial | Direct-transfer clusters; skip CEX/burn. |
| Whale *behaviour* vs alerts | Done | Accumulation/distribution/mixed/rotation/dormant. Empty = UNKNOWN. |
| Baseline 1h/6h/90d + MAD/percentiles | Done | Windows 1h/6h/24h/7d/30d/90d. MAD + p25/p75. Short windows MISSING until enough points. |
| Risk 2.0 (drivers + counter-evidence) | Partial | Multi-factor token risk; no SAFE from score. |
| Scenario / what-if / sensitivity | Done | Twin metric shocks. Banner NOT A PREDICTION. Price not simulated. |
| Twin-to-twin simulation | Partial | Compare A vs B is structural, not a cloned forecast. |
| Research orchestrator + Investigate Next | Done | Copilot + Twin evidence. |
| Ecosystem Copilot | Done | `POST /v2/copilot` + Twin UI. |
| Twin UI 2D / 3D / replay | Done | 2D + isometric 3D projection of the same graph. SSE live list. |
| Alerts 2.0 (lifecycle, regime, evidence conflict) | Partial | Lifecycle transition + price/freshness/liquidity. |
| Daily *ecosystem* brief | Done | `GET /v2/briefing` + `/briefing`. |
| `/v2` Twin API + SSE | Done | Twin resources + `GET /v2/events`. |
| Graph store / Timescale | Temporal Postgres done; Timescale **not yet** | [TIMESERIES_BENCHMARK.md](TIMESERIES_BENCHMARK.md). Neo4j not default. |
| Stripe / white-label / watchlist silos | Done (scaffold) | Branding + `/watchlist` + client filters. Server isolation when `AUTH_REQUIRED`. Stripe webhook if `STRIPE_*`. |
| Solana / social firehose / live LLM | Adapter/port done; LLM blocked | Solana if RPC. Firehose if licence. |

## Technical debt (do not expand)

- Queue kinds complete as no-ops (ledger, not full job bodies)
- CEX address list public and incomplete (confidence capped)
- Postgres down → degraded (honest); Settings billing/queue empty
- Next `/v2` rewrite can buffer SSE — EventSource hits the API origin directly
- RLS only on usage/queue; token intel stays shared on purpose

## Never “fill the gap” by faking

No invented social, wallets, relations, Twin animations without events, price forecasts, EXECUTE, fake SSO users, fake Stripe.
