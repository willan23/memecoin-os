# Security, RBAC, privacy

## Principles

Zero-trust between tenants (schema ready). Least privilege. No user trading keys. Encryption in transit (TLS in deploy). Secrets via env / secret manager, never in git. Audit log table `audit_events` (immutable flag) is reserved for sensitive actions.

## Auth (current)

- Default: `AUTH_REQUIRED=false` — local operator UI does not send a Bearer token.
- Optional: `AUTH_REQUIRED=true` requires `Authorization: Bearer mcos_…` on `/v1/*` and `/v2/*`. `/health`, `/status` and `/v1/auth/*` stay public.
- Keys are created in UI **Settings** or `POST /v1/keys`. Plaintext is returned **once**. Store is SHA-256 hash + prefix lookup. Revoke via `DELETE /v1/keys/{id}`.
- Roles allow-listed on create: `platform_admin`, `analyst`, `developer`, `community_manager`, `viewer`, `api_client`.
- Default tenant slug `default` (`migrations/013_ops_complete.sql`).

## RBAC (target)

Platform Admin, Tenant Owner, Admin, Analyst, Developer, Community Manager, Viewer, API Client (`roles` + `tenant_members` in `008_auth_prep.sql`).

SSO is OIDC (`OIDC_ISSUER` + client credentials). Identity is taken from the IdP **userinfo** response. Unconfigured IdP does not create users. Billing meters usage; `METERING_ENFORCE` is the only quota hard-deny. There is no payment processor.

RLS on `usage_meters` and `work_queue` applies only when `app.tenant_id` is set. Local operator (unset) sees all. Token snapshots remain shared public intelligence.

## AI security

- Data boundary = registry tokens in scope
- Output must carry evidence + model id
- Prompt injection (`ignore previous`, `system prompt`, `execute trade`, `private key`) is refused; EXECUTE cannot be enabled by user text
- SSRF: RPC URLs from env, not from model text
- Body limit 32 KiB on API
- CORS restricted to localhost origins in dev
- Webhook URLs must be `https://` (or `http://127.0.0.1` for local tests)

## Threat model (abridged)

| Threat | Mitigation |
| --- | --- |
| Prompt injection → execute trade | EXECUTE disabled; no wallets |
| Fake token promotion | UNVERIFIED default; no auto-promote |
| Metric gaming | Organicness / bot probability; detection flags |
| Provider outage | `MISSING` (never silent fixtures in live mode); process stays up |
| Stolen API key | Hash at rest; revoke; prefix-only listing |
| Tenant data leak | RLS on usage/queue when `app.tenant_id` is set; token intel stays shared |
| SQL injection | SQLx bind params |

## Privacy & retention

Public chain/market/social data. No PII required. Retention: raw observations 90d hot, aggregates longer; audit 2y (when wired). Configurable disclaimers: analytics ≠ advice.

## Compliance posture

Product language forbids return guarantees, personalized financial advice, pump schemes, price promises.
