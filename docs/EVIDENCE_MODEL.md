# Evidence model

**Now:** every external fact has provenance; research cites lexical chunks from snapshots; injection and price questions are refused.

**Derived claims (live):** `GET /v1/ecosystems/{id}/claims` and MCP `get_claims` emit SUPPORTS / CONTRADICTS / INSUFFICIENT from the current snapshot layers (market, liquidity, holders, social firehose, development). `VERIFIED implies SAFE` is always CONTRADICTS. INSUFFICIENT is not a quiet market.

**Persisted (latest only):** on snapshot refresh, `jobs.claims_persist` writes `evidence_claims` (migration `024`) — one row per `(token_id, claim_id)`. The GET `claims` array is still derived (source of truth). `persisted` / `persisted_count` is the last stored copy. Not a bitemporal graph. No CORROBORATES / INVALIDATES yet.

**Target:** bitemporal claim relations with evidence ids (SUPPORTS / CONTRADICTS / CORROBORATES / INVALIDATES). Intelligence output stays Observation + Claim + Evidence + Confidence + freshness — not a score alone.

Raw payloads (when `jobs.raw_store`) stay content-hashed for audit. No invented sources.
