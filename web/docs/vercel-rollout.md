# Vercel Rollout: Web Registration Per-User Provisioning

Deploy-order requirement for the `web-registration-provisioning` change. **Set the three Turso env vars in Vercel BEFORE deploying** — the register action fails closed by design (R6/D6): without them, registration returns "Registro no disponible temporalmente" and no account is created.

## Required env vars (all three, production)

| Var | Source | Purpose |
|-----|--------|---------|
| `TURSO_API_TOKEN` | `web/.env` (already set) | Turso Platform API auth — creates the per-user database + auth token |
| `TURSO_ORG` | `web/.env` (already set) | Organization slug for `POST /v1/organizations/{org}/databases` |
| `TURSO_GROUP` | `web/.env` (already set) | Group the per-user databases are created in |

Values are already present in the gitignored `web/.env` — copy them from there. Never commit them.

## Steps

1. **Install / authenticate the Vercel CLI** (from `web/`):

   ```bash
   bunx vercel whoami        # if this errors, run: bunx vercel login
   ```

2. **Link the project** (only if `web/.vercel/project.json` does not exist):

   ```bash
   bunx vercel link          # select the Academix web project
   ```

3. **Add the three vars to the production environment** (one per line in the prompt; choose "Production"):

   ```bash
   bunx vercel env add TURSO_API_TOKEN production
   bunx vercel env add TURSO_ORG production
   bunx vercel env add TURSO_GROUP production
   ```

   Or, if the project is already linked, non-interactively:

   ```bash
   bunx vercel env add TURSO_API_TOKEN production <<< "$(grep '^TURSO_API_TOKEN=' web/.env | cut -d= -f2-)"
   bunx vercel env add TURSO_ORG production <<< "$(grep '^TURSO_ORG=' web/.env | cut -d= -f2-)"
   bunx vercel env add TURSO_GROUP production <<< "$(grep '^TURSO_GROUP=' web/.env | cut -d= -f2-)"
   ```

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

## Rollback

- Code: revert the register wiring commits (`web/src/actions/register.ts`, `web/src/pages/auth/register.astro`, `web/src/lib/provisioning.ts`, `web/migrations/per-user/`).
- Env: `bunx vercel env rm <VAR> production` if the feature must be disabled.
- Residual orphan per-user databases (from failed runs) are removed via the Turso API/CLI.
