# `@gnegro/memecoin-os-sdk`

TypeScript client for the **MemeCoin OS** REST API.

- Live: [memecoin-os.web.app](https://memecoin-os.web.app)
- Repo: [github.com/willan23/memecoin-os](https://github.com/willan23/memecoin-os)
- npm: [npmjs.com/~gnegro](https://www.npmjs.com/~gnegro)
- API docs: [API.md](../../docs/API.md)

## Install

```bash
npm install @gnegro/memecoin-os-sdk
```

## Usage

```ts
import { MemeCoinOsClient } from "@gnegro/memecoin-os-sdk";

const client = new MemeCoinOsClient({
  baseUrl: "https://memecoin-os-api-763598651987.europe-west1.run.app",
});

await client.health();
await client.overview();
await client.token("pepe");
await client.discovery();
await client.ask("What should the community investigate?", "pepe");
```

Default `baseUrl` is `http://127.0.0.1:8080` (local API).

License: Apache-2.0
