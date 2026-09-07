# Crypto Ecosystem OS — Phase 4 AI research

Additive. EXECUTE off. The model cannot create evidence. No price targets.

## Pipeline

```
question → intent → lexical RAG (structured snapshots) → specialist agents
        → citation check → research-agent/v1 report
```

Specialists: market, on-chain, risk, development, community, narrative.

Community without a licensed firehose returns **INSUFFICIENT_EVIDENCE** (mentions are not 0).

RAG is complementary. Source of truth remains Postgres + snapshots.

Forecasts exist only for non-price series (`volume_24h_usd`, …) as naive last-value ± 1.5σ. `price_*` is refused.

## API

`POST /v1/research` `{ question, token_id? }`  
`GET /v1/tokens/{id}/research.md`  
`GET /v1/tokens/{id}/research.json`  

`POST /v1/ai/query` now returns the same grounded report as markdown.

PDF is the markdown (print). No binary generator.

## Eval (unit)

Injection refused · price refused · execute false · social missing · citations must match corpus.

Kill switch: `FEATURE_AI_RESEARCH=false` (flag `plane.ai`). No live LLM is called in this phase.
