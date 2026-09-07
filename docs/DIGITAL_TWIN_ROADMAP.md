# Digital Twin roadmap

Crypto OS Phases 1–5 are **done**. This wave implements Twin surfaces incrementally. `/v1` stays compatible. No big-bang rewrite.

## Already in the repo (do not rebuild)

| Crypto OS | V2 overlap |
| --- | --- |
| Phases 1–2 | Domain events, baselines, indexer, whale/CEX, clusters |
| Phase 3 | Token discovery, verification, narratives, genome clusters |
| Phase 4 | Research agents, lexical RAG, reports, eval |
| Phase 5 | Queue, object hash store, tenants, metering, OIDC, v2 *aliases* |

V2 “Phase 8 productization” is **partially** done (meters, tenants, SSO hook). Stripe, white-label, per-tenant token silos stay later.

## Implementation order (V2 §86)

Crypto OS Phases 1–5 and the Digital Twin vertical slice (V2 Phases 1–7 **plus 3D projection, replay, SSE, compare, whales, briefing**) are in the repo.

### Phase 0 — Audit — done

### Phase 1–7 + closable Phase 8 surfaces — done

Ecosystem domain, Twin now + reconstruct, relational 2D/3D graph, lifecycle, similarity, Twin UI, what-if simulation, copilot, replay, SSE, compare, whale behaviour, daily briefing. `/v1` compatible. Flags `digital_twin`, `digital_twin_3d`. Migration `019`.

### NEXT_WAVE — applied 2026-09-05

See **[NEXT_WAVE.md](NEXT_WAVE.md)**. White-label, watchlist, WebGL, Solana adapter, temporal edges, Stripe webhook scaffold, firehose port. Timescale **not yet**. Neo4j not default. Firehose live only with a licence.

### 2026-09-06 — claims persist + baselines + watchlist UI

`evidence_claims` latest row (`024`). Baseline windows 1h/6h/24h/90d + MAD/p25/p75. `/watchlist` + client filters. Cloud Run `memecoin-os-api-00024-v8w`. Operator leftovers: [NEXT_STEPS.md](NEXT_STEPS.md).

### Phase 8 — Productization remainder

Stripe only with a real secret in `.env`. White-label branding. Watchlist isolation. SSE is in-repo (`GET /v2/events`).

## Absolute priority if time is short (§87)

1. Ecosystem domain  
2. Twin state (now)  
3. Historical reconstruction  
4. Evidence graph  
5. Ecosystem discovery  
6. Lifecycle  
7. Genome 2.0 / fingerprint  
8. Similarity  
9. Research on Twin  
10. 3D  
11. Simulation  

3D is visual differentiation. The product is the **model**.

## Definition of done (track; not all tomorrow)

See V2 §96. Closable Twin surfaces compile, tests, `/v1` compatible, docs updated. Remainder: [NEXT_WAVE.md](NEXT_WAVE.md).

## Hard no

Autonomous trading · fake data as live · price “will be X” · decorative Twin motion · auto-VERIFIED · RPC URLs from prompts · rewrite of working core modules.
