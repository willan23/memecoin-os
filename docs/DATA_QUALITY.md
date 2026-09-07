# Data quality

Keep: duplicate, stale, missing, anomaly, source disagreement (price divergence > 25% → `DATA_CONFLICT`). UI shows confidence. Tiny prices are not scientific notation.

Add with Twin: evidence conflict, temporal inconsistency, provider degradation. Never hide a hole with a realistic zero.

History CSV excludes `SIMULATED`. Reconstruction must not use rows after the requested timestamp.
