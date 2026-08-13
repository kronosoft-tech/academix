# Vercel Rollout: Web Registration Per-User Provisioning

Deploy-order requirement for the `web-registration-provisioning` change. **Set the three Turso env vars in Vercel BEFORE deploying** — the register action fails closed by design (R6/D6): without them, registration returns "Registro no disponible temporalmente" and no account is created.

## Required env vars (all three, production)

| Var | Source | Purpose |
|-----|--------|---------|
| `TURSO_API_TOKEN` | `web/.env` (already set) | Turso Platform API auth — creates the per-user database + auth token |
| `TURSO_ORG` | `web/.env` (already set) | Organization slug for `POST /v1/organizations/{org}/databases` |
| `TURSO_GROUP` | `web/.env` (already set) | Group the per-user databases are created in |

Values are already present in the gitignored `web/.env` — copy them from there. Never commit them.

## Payment env vars (Mercado Pago)

| Var | Source | Purpose | Failure mode |
|-----|--------|---------|--------------|
| `MP_WEBHOOK_SECRET` | Mercado Pago dashboard (webhook secret — distinct from the access token) | Required for webhook `x-signature` verification (HMAC-SHA256 over `id:<data.id>;request-id:<x-request-id>;ts:<ts>;`) | Absent → the webhook responds 500 and Mercado Pago retries. `GET /api/payments/verify-mercadopago` is the dashboard-side persistence fallback: it records approved payments even when the webhook keeps failing |
| `MP_ACCESS_TOKEN` | Mercado Pago dashboard (access token) | Authenticates all MP API calls: Checkout Pro preference creation, webhook `getPayment`, and the `verify-mercadopago` endpoint (its direct dependency) | Absent → preference creation and payment verification fail (checkout cannot be created; redirects cannot be verified) |

Same rule as the Turso vars: values live in the gitignored `web/.env` — copy them into the Vercel production environment and never commit them. Add them with `bunx vercel env add <VAR> production` before deploying.

## Steps

1. **Install / authenticate the Vercel CLI** (from `web/`):

   ```bash
   bunx vercel whoami        # if this errors, run: bunx vercel login
   ```

2. **Link the project** (only if `web/.vercel/project.json` does not exist):

   ```bash
   bunx vercel link          # select the Academix web project
   ```

3. **Add the env vars to the production environment** (one per line; choose "Production"):

    ```bash
    bunx vercel env add TURSO_API_TOKEN production
    bunx vercel env add TURSO_ORG production
    bunx vercel env add TURSO_GROUP production
    bunx vercel env add MP_WEBHOOK_SECRET production
    ```

    Or, if the project is already linked, non-interactively:

    ```bash
    bunx vercel env add TURSO_API_TOKEN production <<< "$(grep '^TURSO_API_TOKEN=' web/.env | cut -d= -f2-)"
    bunx vercel env add TURSO_ORG production <<< "$(grep '^TURSO_ORG=' web/.env | cut -d= -f2-)"
    bunx vercel env add TURSO_GROUP production <<< "$(grep '^TURSO_GROUP=' web/.env | cut -d= -f2-)"
    bunx vercel env add MP_WEBHOOK_SECRET production <<< "$(grep '^MP_WEBHOOK_SECRET=' web/.env | cut -d= -f2-)"
    ```

    > `MP_WEBHOOK_SECRET` is the MP Webhooks "Secret key" from the MP Dashboard (App → Webhooks → Secret key) — **not** `MP_ACCESS_TOKEN`.

4. **Verify before deploying**:

   ```bash
   bunx vercel env ls production
   ```

   All three vars must be listed. **Do not deploy until they are.**

5. **Deploy**:

   ```bash
   bunx vercel --prod
   ```

## Post-deploy smoke check

1. Open the production register page (`/auth/register`).
2. Register with a unique email, password, academy name and matching confirmation.
3. Expect a redirect to `/dashboard` showing "Panel de Control" — **not** the "Error de conexión" block and **not** "Registro no disponible temporalmente".
4. If registration fails closed, re-check `bunx vercel env ls production` — a missing var reproduces exactly that symptom.
5. **MP webhook post-fix check**: After deploying with `MP_WEBHOOK_SECRET` set, POST to `/api/webhooks/mercadopago` without an `x-signature` header. Expect **401 "Invalid signature"** — not 500. (Before the fix, the missing secret returned 500; after adding the var, signature failures return 401 as designed.) A real MP notification with a valid `x-signature` returns 200 and processes the payment.

## Rollback

- Code: revert the register wiring commits (`web/src/actions/register.ts`, `web/src/pages/auth/register.astro`, `web/src/lib/provisioning.ts`, `web/migrations/per-user/`).
- Env: `bunx vercel env rm <VAR> production` if the feature must be disabled.
- Residual orphan per-user databases (from failed runs) are removed via the Turso API/CLI.
