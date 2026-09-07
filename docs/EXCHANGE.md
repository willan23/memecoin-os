# Exchange Intelligence

Additive public API + embed card for listing pages. Observable state only. Not a trade signal. **VERIFIED ≠ SAFE.** `execute: false` on every envelope.

## Live

- UI: https://memecoin-os.web.app/exchange
- Embed: `https://memecoin-os.web.app/embed/{id}/`
- Widget: `https://memecoin-os.web.app/widget.js` → `MemeCoinOS.embed({ id, target })`
- Envelope routes under `/v1/ecosystems*` and `/v1/health`

## Envelope

```json
{
  "data": {},
  "timestamp": "…",
  "freshness": "live",
  "confidence": 0.7,
  "evidence": ["…"],
  "powered_by": "MemeCoin OS",
  "execute": false,
  "note": "Exchange Intelligence API. Observable state only. …"
}
```

A blank field is not a zero. Organicness / mention velocity stay `"MISSING"` or `null` without a firehose licence.

## Routes

| Method | Path |
| --- | --- |
| GET | `/v1/health` |
| GET | `/v1/ecosystems` |
| GET | `/v1/ecosystems/{id}` |
| GET | `/v1/ecosystems/{id}/asset` |
| GET | `/v1/ecosystems/{id}/twin` |
| GET | `/v1/ecosystems/{id}/risk` |
| GET | `/v1/ecosystems/{id}/narratives` |
| GET | `/v1/ecosystems/{id}/genome` |
| GET | `/v1/ecosystems/{id}/similar` |
| GET | `/v1/ecosystems/{id}/history?range=30d` |
| GET | `/v1/ecosystems/{id}/claims` | Derived live + `persisted` latest rows |

MCP wraps the same facts: [MCP.md](MCP.md).
