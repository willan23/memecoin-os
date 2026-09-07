# Firebase (project `memecoin-os`)

Public URLs:

- https://memecoin-os.web.app
- https://memecoin-os.firebaseapp.com

Stripe webhook (already in Dashboard):

```
https://memecoin-os.web.app/v1/billing/stripe/webhook
```

Hosting serves the Next export (`apps/web/out`), including `/watchlist`. `/v1/*`, `/v2/*`, and `/health` rewrite to Cloud Run service `memecoin-os-api` in `europe-west1` (`https://memecoin-os-api-763598651987.europe-west1.run.app`, revision **`memecoin-os-api-00024-v8w`** as of 2026-09-06). Without that service the webhook is a 404. Do not deploy this API to `varejo-auditor-745a9`.

## Deploy

```
cd apps/web && npm run export
firebase deploy --only hosting
gcloud run deploy memecoin-os-api --source . --region europe-west1 --allow-unauthenticated --quiet --project=memecoin-os
```

Always `--project=memecoin-os`. The gcloud default project may be something else — do not create `memecoin-os-api` elsewhere.

Cloud Run listens on `0.0.0.0:$PORT`. Postgres is Cloud SQL `memecoin-os-pg` (`db-f1-micro`, Enterprise, `europe-west1`) — about $12/month with 10 GB. Connection is the Cloud SQL Unix socket, not a public `0.0.0.0/0` rule. Redis is still unset. Stripe secrets stay on Cloud Run / Secret Manager — never in Hosting or chat.

Local API stays `http://127.0.0.1:8080`. Use `stripe listen --forward-to http://127.0.0.1:8080/v1/billing/stripe/webhook` when developing.

Web client config lives in `apps/web/.env.local` (`NEXT_PUBLIC_FIREBASE_*`). Restrict the web apiKey by HTTP referrer to `memecoin-os.web.app` and `localhost`.
