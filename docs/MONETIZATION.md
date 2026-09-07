# Monetization

Architecture exists (Phase 5). **No charges** until a real processor secret is configured.

| Plan | Intent | Limits (code) |
| --- | --- | --- |
| free | Limited API / research | 1k API, 20 AI, 5 research / day |
| pro / research / growth | Higher caps | See `crates/core/src/billing.rs` |
| enterprise / api | Unlimited (`-1`) | Meter still recorded |

`METERING_ENFORCE=false` by default (record only). `402` when enforce is on and over quota.

V2 conceptual SKUs (Developer / Startup / Business / Enterprise, white-label) map onto these plans later. Do not invent paid subscribers in the database.

## Connect Stripe (operator — not chat)

Yes: use **your** Stripe account. Do not paste secrets into a prompt.

Operator account (not a secret): `acct_1TQsIIGT8Pzxsiro`. Dashboard: https://dashboard.stripe.com/acct_1TQsIIGT8Pzxsiro/dashboard  
Override with `STRIPE_ACCOUNT_ID` if needed.

1. Stripe Dashboard → Developers → API keys (Test or Live).  
2. Put `STRIPE_SECRET_KEY`, `STRIPE_PUBLISHABLE_KEY`, `STRIPE_WEBHOOK_SECRET` in local `.env` (gitignored).  
3. Price ids map onto `PlanId` in `billing.rs`. Catalog draft (EUR / month, same ladder as Autopilot on this account — change in Dashboard if you want other amounts; a new amount is a new `price_…`):

| Env | Plan | Draft amount |
| --- | --- | --- |
| `STRIPE_PRICE_PRO` | pro | €199 |
| `STRIPE_PRICE_RESEARCH` | research | €299 |
| `STRIPE_PRICE_GROWTH` | growth | €399 |
| `STRIPE_PRICE_API` | api | €499 |
| `STRIPE_PRICE_ENTERPRISE` | enterprise | €999 |

`free` has no Price. Do not reuse WGF Autopilot product ids.

4. Webhook: `POST /v1/billing/stripe/webhook`. Public URL when hosted: `https://memecoin-os.web.app/v1/billing/stripe/webhook` ([FIREBASE.md](FIREBASE.md)). Plan changes only from a verified signature.

`GET /v1/billing` includes `stripe.account_id`, `stripe.state` (`connected` | `keys_missing`). Settings shows **conta ligada** / **em falta**. No charges until secrets exist.
