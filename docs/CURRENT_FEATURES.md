# Current features

Product: **crypto ecosystem intelligence** with a Digital Twin of observed state. Not a price tracker.

## Works (live or honest MISSING)

- YAML registry + EVM onboard (UNVERIFIED; allow-listed chains)
- CoinGecko market + DexScreener liquidity / secondary price; consensus + `DATA_CONFLICT`
- Holder *counts* (Ethplorer ETH, throttled + last-good carry; Etherscan optional)
- GitHub development when an official `github` URL exists (registry or project desk)
- Official https links from DexScreener token `info` (not a firehose)
- Derived claims: SUPPORTS / CONTRADICTS / INSUFFICIENT (`/v1/ecosystems/{id}/claims`). Latest row persisted in `evidence_claims` on refresh. Token page Claims tab. Not a bitemporal graph.
- Social firehose: schema only → not connected (no licence)
- First-party community chat LIVE (`/community`, rooms per token)
- Project claim + official links (label, not VERIFIED)
- Exchange widget + REST envelope + MCP (HTTP + stdio)
- Health / risk / genome with skip + renormalize; concentration UNKNOWN without top-holder sample
- Overview Price column (tiny prices, no scientific notation)
- History 7d/30d/90d + CSV (simulated excluded)
- Compare, rankings, alerts + Discord/Telegram, token briefing 3/2/3
- Settings: flags, keys, refresh-all, plan/usage, queue, SSO status, tenants, watchlist (symbol/name)
- Watchlist page `/watchlist`; Overview + Ecosystems “watchlist only” client filter
- Grounded research (`POST /v1/research`); no price targets; EXECUTE false
- Domain events, Dex pools, wallet classes, baselines 1h/6h/24h/7d/30d/90d (mean/std/z + MAD + p25/p75; MISSING until enough live points)
- ERC-20 transfer cursor (RPC or Etherscan tokentx)
- Whale/CEX events + clusters (hubs skipped; not identity)
- Whale behaviour aggregate (empty = UNKNOWN, not “zero whales”)
- Discovery of DEX counterparts (never auto-VERIFIED)
- Narrative radar (social velocity MISSING)
- Genome clusters (name unused)
- Queue, local object store (off by default), tenant/plan meters, OIDC when configured
- Digital Twin: Ecosystem entity, TwinState now + reconstruct, 2D + WebGL 3D graph, replay, compare, lifecycle, similarity, what-if, copilot, daily ecosystem brief, SSE `twin.updated`
- White-label branding, tenant watchlist (lists only), Solana adapter, temporal graph edges, Stripe webhook scaffold, licensed social port
- `/v2` Twin resources + aliases of a few v1 routes

## UI routes

`/` overview · `/watchlist` · `/twin` · `/twin/[id]` · `/twin/compare` · `/briefing` · `/simulations` · `/tokens` · `/tokens/[id]` · `/projects` · `/projects/[id]` · `/community` · `/exchange` · `/embed/[id]` · `/compare` · `/rankings` · `/alerts` · `/discovery` · `/narratives` · `/intelligence` · `/onboard` · `/settings` · `/status`

## Blocked externally

Production LLM gateway. Social firehose live data (port exists). Timescale. Stripe Checkout waits for webhook + Price ids. Discord/Telegram OAuth wait for Cloud Run secrets. Solana/BSC/Base/Arb holders stay blank without Etherscan or Solana RPC.
