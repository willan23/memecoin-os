# Digital Twin architecture (target)

The Twin is a **state model**, not a 3D view. Renderer never owns business logic.

```
REAL WORLD → providers → raw (hash) → normalize → validate
    → observations → historical state → DIGITAL TWIN
    → intelligence → simulation → AI research → human decision
```

Never: LLM → invented conclusion.

## Domain (add incrementally in `crates/core`)

```
Ecosystem
  DigitalTwin
    Identity · State · Entities · Relationships
    Metrics · Events · Risks · Narratives
    Lifecycle · Genome · Evidence · SimulationState
```

Reuse `TokenSnapshot`, domain events, baselines, clusters, research. Do not fork a second intelligence core.

## Boundaries (keep)

```
providers → canonical domain → ecosystem → digital twin → intelligence → AI
```

Forbidden: frontend → provider, AI → live provider when a canonical snapshot exists, core → CoinGecko types.

## Temporal engine

`get_twin_state(ecosystem_id, timestamp)` reconstructs the nearest **valid** observations **≤ timestamp**. No future rows. Windows: now, 1h, 6h, 24h, 7d, 30d, 90d, custom.

First implementation: compose existing snapshot tables + last-known metrics. Do not introduce Timescale until a benchmark says so ([NEXT_WAVE.md](NEXT_WAVE.md)).

## Graph

Phase 1: PostgreSQL relational edges (`entity_relationships` + evidence ids). Specialized graph store only if volume demands it.

Edges are temporal (`valid_from` / `valid_to`). Correlation ≠ identity (`POTENTIALLY_ASSOCIATED`).

## Simulation

Clone Twin → apply named assumptions → recalculate → compare. Label **SIMULATION / NOT A PREDICTION**. No price guarantees.

## UI stack

```
Twin Domain → Twin View Model → Graph Projection → 2D (then 3D)
```

3D is optional projection (WebGL). No decorative motion without a real event. Isometric SVG is in-repo; Three.js apply: [NEXT_WAVE.md](NEXT_WAVE.md).

## Flags (when code lands)

`digital_twin` · `digital_twin_3d` · `knowledge_graph` · `ecosystem_discovery` · `ecosystem_lifecycle` · `ecosystem_similarity` · `simulation_engine` · `ai_research` (exists as `plane.ai`) · `advanced_alerts`
