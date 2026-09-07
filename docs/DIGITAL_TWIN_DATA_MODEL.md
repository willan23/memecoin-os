# Digital Twin data model (target)

Additive tables. Postgres remains source of truth. JSONB only where the shape is genuinely variable.

## Ecosystem

`ecosystem_id`, `name`, `slug`, `description`, `primary_chain`, `status`, `lifecycle_phase`, `discovery_score`, `intelligence_score`, `risk_score`, `confidence`, `first_seen`, `last_updated`, `data_freshness`, `verification_status`.

Members (`ecosystem_members`): tokens, contracts, pools, clusters — each with `first_seen`, evidence, status. **Never auto-VERIFIED.**

Seed mapping (bootstrap only, not ticker logic): each registry token can start as a *candidate* ecosystem of one until graph formation says otherwise.

## TwinState

Timestamped slice: market, liquidity, holder, wallet, whale, developer, social, narrative, risk, governance, network + `confidence` + `freshness`.

Each field: CURRENT / HISTORICAL / STALE / MISSING / CONFLICT / UNKNOWN. Absence is not zero.

## Graph

Entities: Token, Wallet, WalletCluster, Contract, Pool, DEX, CEX, Developer, Organization, Community, Narrative, Chain, Event, Ecosystem.

Relationships: HOLDS, TRANSFERRED_TO, FUNDED_BY, TRADED_ON, PROVIDES_LIQUIDITY, DEPLOYED, INTERACTED_WITH, CONNECTED_TO, PART_OF, ASSOCIATED_WITH, PARTICIPATES_IN, MENTIONED_BY, DEVELOPED_BY, GOVERNED_BY — plus POTENTIALLY_ASSOCIATED.

Edge: `relationship_id`, source, target, type, confidence, first_seen, last_seen, evidence_ids, status, valid_from, valid_to.

## Evidence / Claim

`evidence_id`, source, provider, raw_reference, timestamps, freshness, confidence, validation_status, claim, payload_hash.

Links: SUPPORTS / CONTRADICTS / CORROBORATES / INVALIDATES.

**Now:** derived SUPPORTS/CONTRADICTS/INSUFFICIENT + latest row in `evidence_claims`. Not bitemporal. CORROBORATES / INVALIDATES not emitted. See [EVIDENCE_MODEL.md](EVIDENCE_MODEL.md).

## Lifecycle

UNKNOWN → DISCOVERED → EMERGING → FORMATION → GROWTH → EXPANSION → MATURATION → DECLINE → DORMANT.

Transitions: previous, new, trigger, evidence, confidence, timestamp.

## Simulation

`simulations`, `simulation_assumptions`, `simulation_results`. Assumptions explicit. Sensitivity per variable.

## Events (append-only, reuse fingerprint)

Add: EcosystemCreated/Updated, EntityAdded/Removed, RelationshipCreated/Changed, TwinStateUpdated, LifecycleChanged, EvidenceAdded/Invalidated, SimulationCreated/Completed, NarrativeShift, AnomalyDetected, …

Idempotent IDs. Reprocess must not duplicate alerts or edges.
