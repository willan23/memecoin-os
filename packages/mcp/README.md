# `@gnegro/memecoin-os-mcp`

Read-only **MemeCoin OS** Exchange Intelligence MCP (stdio). Forwards JSON-RPC to the live API.

- Live terminal: [memecoin-os.web.app](https://memecoin-os.web.app)
- Source: [github.com/willan23/memecoin-os](https://github.com/willan23/memecoin-os)
- npm: [npmjs.com/~gnegro](https://www.npmjs.com/~gnegro)
- Docs: [MCP.md](../../docs/MCP.md)

EXECUTE is never exposed. Missing sources stay blank.

## Install / run

```bash
npx -y @gnegro/memecoin-os-mcp
```

```bash
npm install -g @gnegro/memecoin-os-mcp
memecoin-os-mcp
```

Optional API base (default = production Cloud Run):

```bash
set MEMECOIN_OS_API=https://memecoin-os-api-763598651987.europe-west1.run.app
memecoin-os-mcp
```

## Cursor MCP

```json
{
  "mcpServers": {
    "memecoin-os": {
      "command": "npx",
      "args": ["-y", "@gnegro/memecoin-os-mcp"],
      "env": {
        "MEMECOIN_OS_API": "https://memecoin-os-api-763598651987.europe-west1.run.app"
      }
    }
  }
}
```

License: Apache-2.0
