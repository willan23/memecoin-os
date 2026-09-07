# Token onboarding

Tokens enter only via:

1. YAML in `tokens/*.yaml`
2. Wizard UI `/onboard` → `POST /v1/tokens` (runtime registry + `tokens` row when Postgres is up)

Rules:

- EVM address: `0x` + 40 hex chars
- Allow-listed chains: ethereum, arbitrum, bsc, base, polygon, optimism, avalanche
- No ticker-specific code in `crates/core`
- Status `UNVERIFIED` is not a safety rating — never auto-promote

Pipeline after publish: snapshot (market, DEX, optional holders, GitHub if URL) → score → risk → events. Manual re-index: **refresh** on the token page or Settings → Refresh all snapshots.

To persist a YAML file for git, add `tokens/{token_id}.yaml` using an existing seed file as the template (`tokens/pepe.yaml`).
