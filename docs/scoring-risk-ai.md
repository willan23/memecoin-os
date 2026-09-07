# Scoring, risk & AI

## Ecosystem Health (`ecosystem-health/1.0.0`)

Weighted observable quality, **not** expected price:

| Component | Weight |
| --- | --- |
| Market health | 15% |
| Liquidity | 12% |
| On-chain activity | 12% |
| Community | 12% |
| Development | 10% |
| Utility | 8% |
| Adoption | 8% |
| Governance | 5% |
| Treasury | 5% |
| Risk (inverted) | 13% |

Every score has `why`, drivers, confidence, algorithm version.

**Skip + renormalize:** a component whose source is `MISSING` gets weight 0; remaining weights are renormalized. Community is skipped without a social firehose. On-chain activity can score from holder count alone; active-holder / transfer zeros are not treated as “no activity” in alerts.

Social momentum uses unique accounts, engagement quality, organicness, bot probability and narrative diversity — not raw post count.

## Genome (`1.0.0`)

Narrative, community, utility, development, liquidity, exchange reach, social momentum, holder retention, whale risk, developer activity, governance, treasury, brand, ecosystem activity, transparency, risk control.

UI: token tab **Genome**. Unmeasured social/retention dimensions may read 0 — that is skip/absent data, not a forecast.

Ecosystem Genome 2.0 + structural fingerprint: [DIGITAL_TWIN_ROADMAP.md](DIGITAL_TWIN_ROADMAP.md) Phase 4. Do not claim two tokens will “repeat performance.”

## Token risk (`token-risk/1.0.0`)

Contract, liquidity, concentration, whale, governance, exchange, social manipulation, development, dependency, market.

Levels: `low` / `moderate` / `high` / `critical` / `unknown`.

Concentration / whale stay **UNKNOWN** until a top-holder sample exists (Ethplorer). Holder count alone must not look like 0% concentration.

Manipulation flags are **detection-only** (wash-like volume vs liquidity, bot engagement, sudden LP removal, concentration).

## Growth engine

Current state → gap vs genome → opportunity (impact, cost, risk, confidence, evidence, expected *ecosystem* metric). Humans implement. Closed loop (measure → learn) is later.

## AI

- Grounded observations: `{ observation, evidence[], confidence, interpretation, risk }`
- Daily briefing per token (Overview + 3 changes + 2 risks + 3 opportunities + evidence)
- Per-ecosystem agent + global super-agent (`/intelligence`)
- Prompt-injection: refuse price-target / pump / EXECUTE language
- Tool/policy: EXECUTE cannot be enabled by model output
- Research pipeline `research-agent/v1`: intent → lexical RAG → specialists → citation check
- Optional live LLM later via `OPENAI_API_KEY`; **not** used in Phase 4 (cannot invent evidence)

AI eval (release gate): groundedness, citation, confidence calibration, policy compliance.

Copilot / Investigate Next / Twin-grounded research: [AI_RESEARCH.md](AI_RESEARCH.md) (V2 Phase 7).
