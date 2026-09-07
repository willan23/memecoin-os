# Data providers

Live mode (`DATA_MODE=live`, default):

| Provider | Used for | If down |
| --- | --- | --- |
| CoinGecko | Price, market cap, volume, 24h change | Those fields `MISSING` / 0 — **never** fixtures |
| DexScreener | Liquidity, **secondary spot price** | Liquidity `MISSING`. If CoinGecko is also down, market price may still be DexScreener |
| GitHub | Development (optional `socials.github`) | Development `MISSING` |
| Ethplorer | Ethereum holder count (1 req / token on `freekey`; top-50 only with a paid key) | Last live count is carried and aged (stale), not replaced with 0. First observation stays blank if every call fails |
| Etherscan v2 | Holder count on other EVM chains | Needs `ETHERSCAN_API_KEY` **and** a plan that includes `tokenholdercount` (Pro / Standard+) plus chain coverage. Free keys stay blank. |
| DexScreener `info` | Official https website / social URLs | Empty list if the pair has no `info` |
| Social firehose | Mentions / bots | Always blank until licence (`FEATURE_SOCIAL` + `SOCIAL_FIREHOSE_*`). Chat is a different plane. |

Price consensus: relative divergence CoinGecko vs DexScreener **> 25%** → `DATA_CONFLICT`. UI still shows a consensus price with a conflict badge.

CoinGecko **429**: wait; quota is 12 refreshes/token/hour. DexScreener price can still populate `price_usd` on overview cards while mcap/volume stay 0.

Ethplorer **429**: retry once + last-good carry. No key on non-ETH: holders stay blank. Transfer, active-holder and CEX flow fields are **not** invented when only a holder count exists. Phase 2 indexes ERC-20 `Transfer` logs when `RPC_*` or `ETHERSCAN_API_KEY` is set.

Simulation mode (`DATA_MODE=simulation`) may use `fixtures.rs` and tags `SIMULATED`. Simulated rows are excluded from historical series.

Feature flags: `FEATURE_COINGECKO`, `FEATURE_DEXSCREENER`, `FEATURE_GITHUB`, `FEATURE_HOLDERS`, plus Postgres `feature_flags` (Settings UI).
