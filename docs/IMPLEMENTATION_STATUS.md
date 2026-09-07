# Implementation status

| Area | State |
| --- | --- |
| Token YAML registry | DONE — AIDOGE, Baby Doge, Kishu, PEPE; no ticker branches in core |
| Live CoinGecko market | DONE |
| DexScreener liquidity + secondary price | DONE — fills `price_usd` when CoinGecko 429s |
| Overview / Ecosystems **Price** column | DONE — tiny prices formatted with leading zeros |
| Price consensus / DATA_CONFLICT | DONE |
| DataState LIVE/MISSING/CONFLICT/SIMULATED | DONE |
| GitHub DevProvider | DONE when `socials.github` is set |
| Scoring skip + renormalize | DONE |
| Risk UNKNOWN on missing evidence | DONE — concentration UNKNOWN without top-holder sample |
| Onboard EVM + durable YAML + Postgres | DONE |
| Snapshot refresh job | DONE — Redis lease, job_runs, persist, `POST …/refresh` |
| UI data-state badges + history + CSV | DONE |
| Compare/rankings/ecosystems filters | DONE |
| Genome + Timeline tabs | DONE |
| Settings (flags, keys, refresh-all) | DONE |
| Briefing 3/2/3 + prompt injection tests | DONE |
| REST v1 compatibility | DONE — additive fields only |
| Postgres + Redis | DONE — compose maps 5435 / 6381; in-memory if down |
| Webhooks Discord/Telegram | DONE |
| Status page | DONE — provider table |
| Feature flags (read + write + Settings) | DONE |
| API keys + optional AUTH_REQUIRED | DONE — hashed secrets, default tenant |
| Alert rules + acknowledge | DONE |
| Holder counts (Ethplorer ETH / Etherscan optional) | DONE — transfers / CEX still MISSING |
| Python + TS SDK | DONE |
| Social firehose | DONE port — live only with `FEATURE_SOCIAL` + `SOCIAL_FIREHOSE_*`; else MISSING |
| Production LLM | BLOCKED (gateway + key) |
| Multi-tenant billing / SSO | DONE — Phase 5 + Stripe webhook scaffold (`020`). No fake charges. |
| Solana | DONE adapter — RPC only if `SOLANA_RPC_URL`; holders/transfers MISSING |
| White-label / watchlist / WebGL | DONE — branding, `/watchlist` + Settings labels, client list filter; server isolation when `AUTH_REQUIRED`; WebGL projection + event highlight |
| Domain events + pool discovery + wallet class + baselines | DONE — Phase 1 + `024`: 1h/6h/24h/7d/30d/90d, MAD + p25/p75 (`/baselines`, `/pools`, `/wallets`, `/indexer`) |
| EVM transfer indexer cursor | DONE — Phase 2 (`015`, RPC `eth_getLogs` + Etherscan tokentx fallback) |
| Whale / CEX / clusters | DONE — evidence only; hubs skipped; not identity |
| Discovery / verification / narratives / genome clusters | DONE — Phase 3 (`016`, `/discovery`, `/narratives`) |
| Research agents / lexical RAG / reports / AI eval | DONE — Phase 4 (`017`, `/v1/research`). No live LLM invention. |
| Queue / object store / tenants / billing / SSO / v2 | DONE — Phase 5 (`018`). Shared token data. No fake payments or fake users. |
| Digital Twin / Ecosystem entity / Time Machine | DONE — `019`, `/v2/ecosystems`. Reconstruct no look-ahead. |
| Knowledge graph (temporal) | PARTIAL — 2D relational projection persisted |
| Scenario simulation / what-if | DONE — Twin metric shocks; not a price forecast |
| Twin UI 2D / replay | DONE 2D + 3D projection + replay frames + SSE |
| Copilot / Investigate Next | DONE — `POST /v2/copilot` + Twin UI |
| Whale behaviour | DONE — empty = UNKNOWN |
| Twin compare / briefing | DONE — `/twin/compare`, `/briefing` |
| Exchange Intelligence API + widget | DONE — `/v1/ecosystems*`, `/embed`, `widget.js` |
| Project claim / official links | DONE — `021`/`022`; claim ≠ VERIFIED |
| Community chat + connect port | DONE — `023`; chat LIVE; OAuth only with env |
| Evidence claims (derived + persisted latest) | DONE — `/v1/ecosystems/{id}/claims` + `evidence_claims` (`024`, `jobs.claims_persist`). Live derive; `persisted` is last stored row. Not bitemporal. |
| MCP | DONE — `GET/POST /v1/mcp` + `packages/mcp` stdio. HTTP `get_risk` on `pepe` smoked 2026-09-06. Cursor enable is local. |
| DexScreener official https links | DONE — observed on liquidity snapshot |
| Holder last-good + Ethplorer throttle | DONE — ETH live; other chains need Etherscan |
