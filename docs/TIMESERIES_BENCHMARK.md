# Timescale benchmark (NEXT_WAVE §4)

**Date:** 2026-09-05  
**Decision:** **not yet** — do not convert hypertables.

## Why

V2 §39: Postgres stays source of truth. Timescale only after a written measurement says BRIN is not enough.

This session:

| Probe | Result |
| --- | --- |
| Docker Desktop / compose | Not reliably up (last wave: engine pipe missing). |
| Snapshot row volume | Four seed tokens. History windows 7d/30d/90d on BRIN. |
| `GET /v1/admin/timeseries` | Preview only; now reports `timescale: not_yet`. |

p50/p95 on 90d history was not measured against a loaded Timescale instance because there is no production-scale table and no live Timescale service. Introducing Timescale now would be fashion, not need.

## Re-run when ready

1. `docker compose up -d` (ports 5435 / 6381).  
2. Time `GET /v1/tokens/{id}/history?range=90d` and `GET /v2/ecosystems/{id}/twin/history` (n≥50).  
3. If p95 is unacceptable **and** `market_snapshots` is large, add Timescale to compose and convert those two tables only. Keep the same SQL API.

Neo4j: not in this path. Temporal edges live on `entity_relationships.valid_from` / `valid_to` (migration `020`).
