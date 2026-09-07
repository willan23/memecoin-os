# MemeCoin OS Python SDK

```python
from memecoin_os import MemeCoinOsClient

client = MemeCoinOsClient(
    base_url="https://memecoin-os-api-763598651987.europe-west1.run.app"
)
print(client.overview())
print(client.ask("What should the community investigate?", token_id="pepe"))
```

Optional `api_key=` when `AUTH_REQUIRED=true`.

- Live: https://memecoin-os.web.app  
- Repo: https://github.com/willan23/memecoin-os  
- API: [docs/API.md](../../docs/API.md)  
- TypeScript (npm): `@gnegro/memecoin-os-sdk`
