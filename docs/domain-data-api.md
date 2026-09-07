# Domain, data & API

## Canonical model

Token, Chain, Contract, Pool, Exchange, Wallet, Holder, Transaction, Transfer, Swap, LiquidityEvent, SocialPost, SocialMention, DeveloperEvent, NewsEvent, GovernanceProposal, TreasuryMovement, AIObservation, Signal, Alert, Recommendation.

Every external fact carries:

```
source · timestamp · freshness · confidence · validation_status · provider · raw_reference
```

Plus roll-up `data_state`: `live` / `recent` / `stale` / `missing` / `conflict` / `simulated`.

## Token schema

See `tokens/*.yaml`. Required: `token_id`, `symbol`, `name`, ≥1 chain with contract. Optional: socials, market provider id, feature flags, supply.

Verification levels: `unverified` → `data_verified` → `contract_verified` → `ecosystem_verified` → `community_verified`. **Verified ≠ safe.**

Discovery statuses (token counterparts, Phase 3): `discovered` / `watchlist` / `high_risk`. New pairs are never auto-promoted. **Ecosystem** discovery (V2) is a later entity — see [ECOSYSTEM_MODEL.md](ECOSYSTEM_MODEL.md).

## Chain adapter

```rust
trait ChainAdapter {
    fn chain_id(&self) -> &str;
    async fn get_block_height(&self) -> Result<u64>;
    async fn get_token_metadata(&self, contract: &str) -> Result<TokenMetadata>;
    async fn get_balance(&self, wallet: &str, token: &str) -> Result<Balance>;
}
```

EVM family first (Ethereum, Arbitrum, BNB, Base, Polygon, Optimism, Avalanche). Holder counts: Ethplorer (ETH) / optional Etherscan v2. Transfer *cursor* exists when `RPC_*` or Etherscan is set (Phase 2). Not a full mempool/swap indexer.

## REST (v1)

Canonical list: [API.md](API.md). Overview cards include `price_usd`. History is Postgres snapshots (simulated excluded). SDKs: `@memecoin-os/sdk`, `packages/sdk-python`, MCP [MCP.md](MCP.md). Exchange: [EXCHANGE.md](EXCHANGE.md).

## Data quality

- Duplicate / stale / missing / anomaly / source disagreement
- Price consensus with `DATA_CONFLICT` when relative divergence > 25%
- UI must show confidence; never fake precision
- Tiny memecoin prices are formatted with leading zeros (not `$3.71e-6`)
- Unmeasured holder activity (transfers, CEX) stays 0 with an explicit MISSING hint when only a count exists

## Historical replay

Live mode: series from `market_snapshots` / scores when Postgres is up. Simulation mode: deterministic fixture series tagged `SIMULATED` and **excluded** from history CSV. Twin reconstruction (`get_twin_state` at timestamp T, no look-ahead) is V2 Phase 2 — [DIGITAL_TWIN_ROADMAP.md](DIGITAL_TWIN_ROADMAP.md).
