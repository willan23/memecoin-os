# Publish (GitHub + npm)

| Surface | Target |
| --- | --- |
| Source | https://github.com/willan23/memecoin-os |
| npm user | https://www.npmjs.com/~gnegro |
| Packages | `@gnegro/memecoin-os-mcp` · `@gnegro/memecoin-os-sdk` |
| Product | https://memecoin-os.web.app |

## Never publish

- `.env`, `client_secret*.json`, Stripe / operator / Discord secrets
- `apps/web/out`, `node_modules`, `target/`

## npm (first time)

```bash
npm login
# account: gnegro

cd packages/mcp
npm publish --access public

cd ../sdk
npm install
npm publish --access public
```

Scoped packages under `@gnegro/` appear on your npm profile after publish.

## GitHub

```bash
git push -u origin main
```

Repo settings → About: set homepage to `https://memecoin-os.web.app`, topics `crypto`, `memecoin`, `mcp`, `rust`, `typescript`.

Optional: feature the repo on [github.com/willan23](https://github.com/willan23/).
