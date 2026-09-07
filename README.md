# MemeCoin OS

**Crypto ecosystem intelligence** — not a price tracker.

Token-agnostic Digital Twin + intelligence layer for memecoin (and broader token) ecosystems: registry → market/liquidity/holders → health & risk → discovery → research → Twin / Exchange surfaces.

| | |
| --- | --- |
| **Live terminal** | [memecoin-os.web.app](https://memecoin-os.web.app) |
| **GitHub** | [github.com/willan23/memecoin-os](https://github.com/willan23/memecoin-os) |
| **npm** | [@gnegro](https://www.npmjs.com/~gnegro) · `@gnegro/memecoin-os-mcp` · `@gnegro/memecoin-os-sdk` |
| **License** | Apache-2.0 |

```bash
# MCP (agents / Cursor)
npx -y @gnegro/memecoin-os-mcp

# TypeScript SDK
npm install @gnegro/memecoin-os-sdk
```

## What it is / is not

**Is:** ecosystem health & risk scores, discovery of DEX counterparts, research reports, Digital Twin views, Exchange REST + MCP, project claim desk (label ≠ VERIFIED), Google SSO, metered plans.

**Is not:** price forecasts, pump coordination, wash-trading tools, auto-VERIFIED for paying customers, trading bots, or EXECUTE agents.

Missing provider data stays blank / `MISSING` / `UNKNOWN`. **Paying never changes a Platform Score.**

## Architecture

```
apps/web              Next.js terminal (React + Tailwind)
crates/api            REST API (Axum) — Cloud Run in production
crates/core           domain, registry, providers, scoring, risk, AI
tokens/               YAML token definitions (source of truth)
packages/sdk          TypeScript client → @gnegro/memecoin-os-sdk
packages/sdk-python   Python client
packages/mcp          MCP stdio → @gnegro/memecoin-os-mcp (HTTP also at /v1/mcp)
docs/                 architecture, API, security, status
```

Modular monolith. Providers and chains are adapters. Product logic never hard-codes a ticker in core.

## Production

- **UI:** https://memecoin-os.web.app  
- **API:** Cloud Run `memecoin-os-api` (GCP project `memecoin-os`, `europe-west1`)  
- **Auth:** Google OIDC when configured · operator writes use `OPERATOR_API_KEY`  
- **Billing:** Stripe webhook updates plan; Checkout UI still early-access  

## Quick start (local)

```bash
# optional — Postgres 5435 / Redis 6381
docker compose up -d

cargo run -p memecoin-os-api
```

```bash
cd apps/web && npm install && npm run dev
```

- API: http://127.0.0.1:8080/health  
- UI: http://localhost:3000  

Copy `.env.example` → `.env`. Never commit secrets (`client_secret*.json`, Stripe keys, operator key).

`DATA_MODE=live` (default): CoinGecko + DexScreener. Without Docker the API runs in-memory (degraded: no durable history / webhooks / keys).

## Packages

### MCP — `@gnegro/memecoin-os-mcp`

Read-only Exchange Intelligence MCP. Forwards JSON-RPC to the live API. EXECUTE is never exposed.

```bash
MEMECOIN_OS_API=https://memecoin-os-api-763598651987.europe-west1.run.app \
  npx -y @gnegro/memecoin-os-mcp
```

Cursor / MCP config: see [docs/MCP.md](docs/MCP.md) and `.cursor/mcp.json`.

### SDK — `@gnegro/memecoin-os-sdk`

```ts
import { MemeCoinOsClient } from "@gnegro/memecoin-os-sdk";

const client = new MemeCoinOsClient({
  baseUrl: "https://memecoin-os-api-763598651987.europe-west1.run.app",
});
const overview = await client.overview();
```

Python: `packages/sdk-python` (install from repo for now).

## Plans (meters)

| Plan | EUR/mo (Stripe) | API/day | AI/day | Research/day | Watchlist tokens |
| --- | ---: | ---: | ---: | ---: | ---: |
| free | — | 1 000 | 20 | 5 | 10 |
| pro | 199 | 20 000 | 200 | 50 | 50 |
| research | 299 | 20 000 | 500 | 200 | 50 |
| growth | 399 | 50 000 | 500 | 100 | 200 |
| api | 499 | unlimited | unlimited | unlimited | unlimited |
| enterprise | 999 | unlimited | unlimited | unlimited | unlimited |

Quota enforce is off by default (`METERING_ENFORCE`). Soft launch: read the terminal freely; catalog **Track** / claim writes still need the operator key until self-serve onboard ships.

## Product rules

Forbidden: pump coordination, wash trading, fake volume/engagement, price promises.

Scores reflect observable ecosystem quality and activity — not hype.

## Documentation

- [Docs index](docs/README.md) · [API](docs/API.md) · [MCP](docs/MCP.md) · [Exchange](docs/EXCHANGE.md)  
- [Monetization](docs/MONETIZATION.md) · [Security](docs/security.md) · [Status](docs/IMPLEMENTATION_STATUS.md)  
- [Next steps](docs/NEXT_STEPS.md) · [Publish checklist](docs/PUBLISH.md)

## Author

Built by [willan23](https://github.com/willan23/) · npm [gnegro](https://www.npmjs.com/~gnegro)

---

*Early access. Building in public.*
