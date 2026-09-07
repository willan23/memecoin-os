# Next wave — apply 2026-09-01

**Superseded for “what next”:** [NEXT_STEPS.md](NEXT_STEPS.md) (2026-09-06). This file stays as the operator briefing that closed Twin leftovers.

Briefing for tomorrow. Twin Phases 1–7 + closable Phase 8 surfaces are **done**. This wave is the remainder the operator listed. Design briefs stay private. `/v1` stays additive.

Hub: [DIGITAL_TWIN.md](DIGITAL_TWIN.md) · gaps: [CURRENT_GAPS.md](CURRENT_GAPS.md) · status: [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md)

## Operator answers (do not re-ask)

| Question | Decision |
| --- | --- |
| Stripe | Operator connects **later**. Scaffold only. No charges, no fake customers. |
| Can I connect my Stripe account here? | **Yes — via local env, not chat.** Stripe Dashboard → Developers → API keys. Put `STRIPE_SECRET_KEY`, `STRIPE_PUBLISHABLE_KEY`, `STRIPE_WEBHOOK_SECRET` in `.env` (never commit, never paste into a prompt). When those exist, Checkout / Customer Portal / webhooks may turn on. Until then `GET /v1/billing` stays “meters only”. |
| White-label | Apply. Branding + domain per tenant. No invented subscribers. |
| Timescale / Neo4j | Timescale only after a **written benchmark**. Neo4j **not** for fashion — deepen Postgres temporal edges first (V2 §39–40). |
| Social firehose | Apply **only** with a real licence/key. Else stay `MISSING`. |
| Solana | Apply adapter + env RPC. No chain logic in the intelligence core. |
| Per-tenant token silos | Apply **watchlist isolation**, not a forked Twin. Shared snapshots stay shared. |
| 3D WebGL | Apply. Renderer is a projection of the existing Twin graph. |
| Animation of real events | Apply. Motion only from persisted timeline / SSE `twin.updated` / replay frames. |

## What is already in the repo (do not rebuild)

| Surface | Reuse |
| --- | --- |
| Plans / meters / 402 | `billing.rs`, `GET /v1/billing`, `METERING_ENFORCE` |
| Tenants / OIDC / queue | migration `018`, `/v1/tenants`, `/v1/auth/sso` |
| Twin graph + isometric 3D | `twin_graph.rs`, `apps/web/src/components/twin-graph.tsx`, flag `digital_twin_3d` |
| Replay + SSE | `twin_replay.rs`, `GET /v2/events`, EventSource → API origin |
| Social port | `SocialProvider` + `NullSocialProvider` → always `None` |
| Multi-chain trait | `ChainAdapter` + `ChainFamily::Solana` (no adapter body) |
| Time-series | BRIN on `market_snapshots.as_of` / `liquidity_snapshots.as_of` |
| Graph edges | `entity_relationships` (migration `019`) |
| Token intel | **Shared public data** on purpose (Phase 5) |

## Apply order (if time is short, stop after 3)

1. **White-label + watchlist silos** — productization without new vendors  
2. **WebGL 3D + event animation** — visual differentiation on the existing model  
3. **Solana adapter** — if `SOLANA_RPC_URL` is set  
4. **Postgres temporal graph 2.0** — `valid_from` / `valid_to` / evidence ids  
5. **Timescale** — only if the benchmark says hypertables win  
6. **Firehose** — only if a licence + key exist  
7. **Stripe** — operator-owned; hook when secrets exist  

Do not start Neo4j, ClickHouse, or a second Twin core.

---

## 1. Stripe (operator later — connect path)

**Goal:** real processor when the operator drops keys in `.env`. Until then, no Checkout UI that pretends to charge.

**Connect (when ready):**

1. [Stripe Dashboard](https://dashboard.stripe.com) → Developers → API keys.  
2. Add to local `.env` (gitignored):

```
STRIPE_SECRET_KEY=sk_live_…   # or sk_test_… for Test mode
STRIPE_PUBLISHABLE_KEY=pk_…
STRIPE_WEBHOOK_SECRET=whsec_…
STRIPE_PRICE_PRO=price_…      # optional Price ids per plan
```

3. Webhook endpoint (when coded): `POST /v1/billing/stripe/webhook` → `checkout.session.completed`, `customer.subscription.updated|deleted`.  
4. Map Stripe Price → existing `PlanId` (`free` / `pro` / `research` / `growth` / `enterprise` / `api`). Update `tenants.plan` only from verified webhook signatures.

**Hard no:** invent paid rows, fake customers, accept a key from an AI prompt, charge without `stripe` crate + signature check.

**Reuse:** [MONETIZATION.md](MONETIZATION.md), `crates/core/src/billing.rs`.

---

## 2. White-label (V2 §64)

**Goal:** custom branding, custom domain, private workspace chrome. Token facts stay the shared Twin.

**Add**

- `tenants.branding` JSONB: `display_name`, `logo_url` (https only), `accent`, `custom_domain`  
- `GET/PATCH /v1/tenants/{id}/branding` — owner/admin only  
- Settings UI: name, logo URL, accent, domain  
- Shell reads branding for the current tenant (default tenant = current MEMECOIN OS chrome)

**Hard no:** CSS/JS from untrusted URLs, SSRF to fetch logos from `http://` internal hosts, per-tenant *invented* dashboards that hide `MISSING`.

---

## 3. Per-tenant token silos (honest isolation)

**Today:** snapshots, Twin, discovery are **shared**. Isolation is keys / usage / queue / sessions.

**Apply (watchlist, not a fork):**

- Table `tenant_watchlist (tenant_id, token_id, created_at)`  
- `tokens_monitored` meter already exists on the plan — enforce count of watchlist rows  
- Overview / Twin **list** filters to the watchlist when `AUTH_REQUIRED=true`  
- Local operator (`AUTH_REQUIRED=false`) still sees the full registry  
- `GET /v2/ecosystems?scope=watchlist|all`  

**Hard no:** copy snapshots per tenant, pretend two tenants have different PEPE Twins, RLS that hides public market facts from the operator.

---

## 4. Timescale / Neo4j (V2 §39–40)

### Timescale — gated

Postgres remains source of truth.

**Tomorrow morning (required before any hypertable):**

1. With Docker up, measure `GET /v2/ecosystems/{id}/twin/history?range=90d` and `GET /v1/tokens/{id}/history?range=90d` p50/p95 on current BRIN.  
2. Write numbers into this file (or a comment on the PR).  
3. Only if p95 is unacceptable **and** row count justifies it: add Timescale to compose, convert `market_snapshots` / `liquidity_snapshots` to hypertables, keep the same SQL API.

Do not migrate prematurely. Do not introduce ClickHouse in the same day.

### Neo4j — not default

V2: do not add a graph DB for fashion.

**Apply instead:** temporal columns on `entity_relationships` (`valid_from`, `valid_to`, `evidence_ids`, `status`). Reconstruct Twin graph at time T by `valid_from ≤ T AND (valid_to IS NULL OR valid_to > T)`.

Neo4j only if the operator later stands up an instance **and** a written volume argument exists. Not in the default 2026-09-01 path.

---

## 5. Social firehose

**Port already exists.** `NullSocialProvider` returns `None` → `MISSING`. Community scoring already skips + renormalizes.

**Apply only with:**

```
SOCIAL_FIREHOSE_URL=…
SOCIAL_FIREHOSE_KEY=…
FEATURE_SOCIAL=true
```

Implement `LicensedSocialProvider` behind that flag. Persist raw payload hash (`jobs.raw_store`). Mentions, bots, velocity become live **only** from that provider.

**Hard no:** scrape Twitter without a licence, invent mention counts, treat holder/social zeros as “quiet market”, fire `SocialSpikeDetected` on MISSING.

If keys are absent at start of day: skip this item. Leave [DATA_PROVIDERS.md](DATA_PROVIDERS.md) as-is.

---

## 6. Solana (V2 §45)

**Trait exists:** `ChainFamily::Solana`. Core must not grow `if chain == "solana"` branches in scoring/risk/Twin.

**Apply**

- `SolanaAdapter` implementing `ChainAdapter`  
- RPC **only** from `SOLANA_RPC_URL` (never from a prompt)  
- Onboard: allow `chain: solana` + base58 mint in YAML  
- DexScreener already returns Solana pairs — reuse for liquidity when the pair is evidenced  
- Holders/transfers: MISSING until a Solana indexer exists (do not invent)

**Hard no:** ticker-specific Solana logic, EXECUTE, private keys, fake SPL balances.

---

## 7. 3D WebGL (V2 §11)

**Today:** SVG isometric projection of `twin_graph` entities (`x,y,z`). Flag `digital_twin_3d`.

**Apply**

- Three.js (or equivalent) in `apps/web` **only**  
- Input = same `/v2/ecosystems/{id}/twin/graph` JSON  
- No business logic in the renderer  
- Keep 2D + list fallbacks  
- Size = `importance`; color = kind; camera orbit replaces the yaw slider  

**Hard no:** a second graph model, decorative idle motion, WebGL when `digital_twin_3d` is off.

---

## 8. Animation of real events

**Allowed sources (must already exist as evidence):**

- Replay frames from `twin_replay`  
- SSE `twin.updated`  
- Timeline events on the snapshot (`kind`, `at`, `source`)  
- Domain events already persisted  

**Apply**

- Pulse / highlight the node whose `kind` matches the current replay frame or latest SSE payload  
- Slider already drives `highlightKind` — bind WebGL to the same field  
- Empty frames → **no** motion (honest empty, not a looping demo)

**Hard no:** Lottie/ambient particles, interpolated “price fly-ins”, animating MISSING social nodes.

---

## Env checklist (gitignored `.env`)

| Variable | Needed for | If unset |
| --- | --- | --- |
| `STRIPE_SECRET_KEY` + webhook secret | Charges | Meters only |
| `SOLANA_RPC_URL` | Solana adapter | Solana stays reserved |
| `SOCIAL_FIREHOSE_URL` + key | Mentions | Social `MISSING` |
| `DATABASE_URL` / Redis | Persist Twin, replay, whales | Degraded in-memory |
| `OIDC_*` | SSO | Already: “not configured” |

Docker Desktop must be running for Timescale experiments (`5435` / `6381`).

---

## Definition of done (2026-09-05)

- [x] White-label branding persists and paints the shell for the current tenant  
- [x] Watchlist filters Twin/overview when auth is on; operator still sees all  
- [x] WebGL renderer consumes Twin graph JSON; 2D still works  
- [x] Node highlight only when a real event/frame is selected  
- [x] Solana adapter compiles; live calls only if RPC env is set  
- [x] Temporal edges queryable at time T (Postgres)  
- [x] Timescale: **not yet** — [TIMESERIES_BENCHMARK.md](TIMESERIES_BENCHMARK.md)  
- [x] Firehose: `LicensedSocialProvider` only with env; else `MISSING`  
- [x] Stripe: meters-only unless `STRIPE_*` set; webhook verifies signature  
- [x] `cargo test -p memecoin-os-core -p memecoin-os-api --lib` green (79 + 8)  
- [x] `/v1` clients unchanged (additive routes only)  

## Hard no (whole day)

Autonomous trading · fake live data · price “will be X” · decorative Twin motion · auto-VERIFIED · RPC/Stripe/firehose URLs from prompts · rewrite of working Twin/core modules · Neo4j without a volume argument · per-tenant forked PEPE snapshots.
