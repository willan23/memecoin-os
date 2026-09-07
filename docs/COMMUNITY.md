# Community

Two planes. Do not mix them.

| Plane | State | Notes |
| --- | --- | --- |
| First-party chat | **LIVE** | Rooms `global` + one per token id. Guest session. SSE `/v2/events` type `community.message`. Empty room = empty, not disconnected. |
| Official links | **LIVE when URL exists** | Registry, project profile, or DexScreener `info` (https only). |
| Mention firehose | **Not connected** | Needs `FEATURE_SOCIAL` + `SOCIAL_FIREHOSE_URL` + `SOCIAL_FIREHOSE_KEY`. Never filled from chat or follower counts. |
| Discord / Telegram OAuth | **Port only** | `DISCORD_CLIENT_*` / `TELEGRAM_BOT_*` on Cloud Run. Unset = buttons wait. Not a substitute for chat. |

## Routes

| Method | Path |
| --- | --- |
| GET | `/v1/community` |
| POST | `/v1/community/session` |
| GET | `/v1/community/rooms` |
| GET/POST | `/v1/community/rooms/{id}/messages` |
| GET | `/v1/connect` |
| GET | `/v1/connect/discord` + `/callback` |
| GET | `/v1/connect/telegram` |
| POST | `/v1/connect/telegram/inbound` |

UI: `/community?room={token_id}`. Header `x-community-session`. Handle is a display name, not identity, not VERIFIED.

Discord Developer Portal (optional chrome): Terms `https://memecoin-os.web.app/terms/` · Privacy `https://memecoin-os.web.app/privacy/`. Interactions / linked-roles URLs stay empty. OAuth scope is only `webhook.incoming`. Redirect: `https://memecoin-os.web.app/v1/connect/discord/callback`.

Schema: `migrations/023_community_connect.sql`.
