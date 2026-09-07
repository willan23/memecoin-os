# Plano de mercado

Design-partner ready without paid explorer/social keys:

- Live market + DEX liquidity; **spot price on Overview / Ecosystems**
- Honest `MISSING` on social firehose and on non-ETH holders without Etherscan key
- Ethereum holder counts via Ethplorer when the public API allows
- History, alerts, webhooks, status (need Docker Postgres for persistence)
- Settings: flags, API keys, refresh-all
- PEPE as fourth tracked ecosystem (plus AIDOGE, Baby Doge, Kishu)

Onda 1 (partial): Ethplorer holders. Transfers / CEX / whale *movements* still MISSING until indexed RPC.

Onda 2: API keys + flags + Settings. Onda 5: metering (no Stripe) + OIDC when issuer is set.

Next commercial surface: Digital Twin (shipped) + [NEXT_WAVE.md](NEXT_WAVE.md) (white-label, watchlist, WebGL, Solana). Stripe: operator connects via `.env` ([MONETIZATION.md](MONETIZATION.md)).

Still blocked until a key: social firehose licence, production LLM gateway. Solana/WebGL/white-label are scheduled, not blocked.

EXECUTE stays off. Scores are not price forecasts.
