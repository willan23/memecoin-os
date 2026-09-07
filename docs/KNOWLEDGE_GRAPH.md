# Knowledge graph

**Now:** ERC-20 transfer clusters; CEX/burn hubs excluded; not identity.

**Target (V2 Phase 3):** temporal relational graph in Postgres. Edges carry confidence, validity window, evidence ids. Use `POTENTIALLY_ASSOCIATED` when the public record is only correlation.

Do not add Neo4j in the first slice (or for fashion). Next apply: temporal `valid_from` / `valid_to` on Postgres edges — [NEXT_WAVE.md](NEXT_WAVE.md). Do not assert “same person”.
