# Simulation

**Now:** none. `forecast.rs` only does naive last-value ±1.5σ on **non-price** series and refuses `price_*`.

**Target (V2 Phase 6):** clone Twin → apply assumptions → recalculate → delta + uncertainty + sensitivity.

Every result: `SIMULATION` / `NOT A PREDICTION` + assumptions. Never a price guarantee. Queue long runs on `work_queue`.
