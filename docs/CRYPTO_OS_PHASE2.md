# Crypto Ecosystem OS — Phase 2 on-chain intelligence

Additive. EXECUTE off. Holder counts are **not** transfers.

## Delivered

| Item | Behavior |
| --- | --- |
| Transfer cursor | `eth_getLogs` Transfer topic when `RPC_*` is set. Window = last `INDEXER_BLOCK_SPAN` (default 200) minus confirmations. Huge gaps skip to tip — no genesis scan. |
| Etherscan fallback | `tokentx` last 100 if `ETHERSCAN_API_KEY` is set and RPC is missing. |
| Whale engine | Events only from indexed transfers + ≥1% holder candidates or public CEX labels. Not a trade signal. |
| CEX flow | Incomplete public hot-wallet list. Confidence capped at 0.68. |
| Clusters | Union-find on transfer edges. **CEX/burn hubs skipped** so they do not merge everyone. Path ≠ identity. |
| Anomaly | Large transfer = outlier vs the current index window (≥5 amounts, z ≥ 2.5). |

## API

`GET /v1/tokens/{id}/transfers`  
`GET /v1/tokens/{id}/whale-events`  
`GET /v1/tokens/{id}/clusters`  
`GET /v1/indexer` — RPC flag + persisted cursors

Flags: `jobs.indexer`, `jobs.whale_engine`, `jobs.wallet_clusters`. Kill switch: `FEATURE_ONCHAIN_INDEXER=false`.

## Not this wave

Swap indexer, discovery/verification, narratives, RAG, billing, Solana, identity claims, price targets.
