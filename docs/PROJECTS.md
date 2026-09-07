# Project desk

A **PROJECT CLAIM** is a label. It is not VERIFIED. Paying never changes a score.

## Live

- List: https://memecoin-os.web.app/projects
- Desk: `/projects/{id}` — official https links, claim/unclaim (operator key), alert catalog

Official URLs merge, in order: project profile → YAML `socials` / website → DexScreener observed https links.

## Routes

| Method | Path | Notes |
| --- | --- | --- |
| GET | `/v1/projects` | Catalog + health/risk |
| GET | `/v1/projects/{id}` | Claim + official + computed verification |
| POST | `/v1/projects/{id}/claim` | `x-operator-key` when configured |
| POST | `/v1/projects/{id}/unclaim` | Operator |
| PATCH | `/v1/projects/{id}/profile` | https only |
| GET | `/v1/projects/{id}/alerts` | Live vs not-connected kinds |

Alert kinds that need a missing source stay not connected (holders/social/narrative/listing/contract). Liquidity/risk fire when those snapshots are live.

Schema: `migrations/021_project_claims.sql`, `022_project_alerts.sql`.
