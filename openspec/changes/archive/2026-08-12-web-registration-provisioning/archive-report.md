# Archive Report: web-registration-provisioning

**Archive date**: 2026-08-12
**Mode**: hybrid (OpenSpec file + Engram) — BOTH backends written
**Archived to**: `openspec/changes/archive/2026-08-12-web-registration-provisioning/`
**Engram**: `sdd/web-registration-provisioning/archive-report`

## Change Summary

Web-registered users never got their per-user Turso database: the register action wrote the shared `users` row and a shared trial row but no `user_databases` row, so `getUserDb` returned `NOT_FOUND` and the dashboard was broken. This change ports the desktop provisioning stack to `web/`: `web/src/lib/provisioning.ts` (slug generation, Turso REST client with 409-retry, migration runner, `provisionUser` orchestrator), `web/migrations/per-user/` (20 byte-identical copies of desktop migrations 001–020), register action/form wiring (`academyName` + `confirmPassword`, 9-step ordering, best-effort DB DELETE cleanup, fail-closed env gating), JWT `dbUrl`/`dbToken`/`academyName` claims (resolving the pre-existing TS2345), and an E2E register→dashboard spec. The shared DB remains the subscription/payment source of truth (R7/B1).

## Intentional Archive with Warnings

Archive proceeds under an authorized non-green verification envelope:

- `verify-report.md` envelope verdict is `fail` — the **mechanical canonical-failure representation** of non-zero declared command exits (`test_exit_code: 1`, `build_exit_code: 2`), NOT a requirement failure. Both non-zero exits come **exclusively** from the R8-approved pre-existing Stripe-era baseline: 7 failing tests ≤ 7 and 7 `tsc` errors ≤ 7, both identical to the pre-change set, zero new from this change.
- Human assessment: **PASS WITH WARNINGS** — 8/8 requirements, 17/17 scenarios compliant, **0 CRITICAL** findings; focused suites for this change exit 0 (`provisioning.test.ts` 38/38, `register-action.test.ts` 12/12).
- The orchestrator confirmed the gatekeeper acknowledged the mechanical fail envelope as the R8-approved baseline and **authorized archive**. No native `reviews/` bundle exists for this change; the gate re-validates after this archive returns (execution mode: auto).

## Implementation

| Commit | Message | Scope |
|--------|---------|-------|
| `8e0e89b` | `chore(web): mirror desktop migrations 001-020 into web/migrations/per-user` | Migration copies (PR 1) |
| `189ec5f` | `feat(web): add Turso provisioning library (slug, REST client, migration runner)` | `provisioning.ts` (PR 2) |
| `38f0c9f` | `feat(web): wire per-user provisioning into register action with env gating and cleanup` | Register action (PR 3) |
| `b66e7d5` | `feat(web): add academyName and confirmPassword to registration form` | `register.astro` (PR 3) |
| `f21285c` | `test(web): e2e registration provisions per-user db and dashboard loads` | E2E + `auth.spec.ts` selector fix (PR 4) |
| `6d209bd` | `docs(web): document Vercel rollout env vars for provisioning` | `web/docs/vercel-rollout.md` (PR 4) |

Delivery: 4 work units, stacked-to-main; all commits local, no PRs opened (orchestrator instruction). No shared-DB schema change. Rollback boundary: revert the 6 commits (register action/form, `provisioning.ts`, `web/migrations/per-user/`, E2E spec, rollout doc); residual orphan DBs removed via Turso API; Vercel vars removable via `vercel env rm <VAR> production`.

## Gates Passed

- **Task completion gate**: `tasks.md` shows **17/17** tasks `[x]` (Phase 1: 3/3, Phase 2: 5/5, Phase 3: 6/6, Phase 4: 3/3), zero unchecked implementation tasks. The "Out of Scope (NOT tasks)" section documents non-task follow-ups without checkboxes. **No stale unchecked implementation tasks in the archived audit trail.**
- **Verification gate**: 8/8 requirements and 17/17 scenarios compliant (spec compliance matrix in `verify-report.md`); **0 CRITICAL** findings; warnings W1 (Vercel rollout documented, not executed), W2 (E2E evidence from apply-progress, not re-run in verify), W3 (server-reported DB name/hostname, desktop parity). CRITICAL-free → no blocker on archive.
- **Review/archive authorization**: gatekeeper acknowledged the mechanical `fail` envelope as the R8-approved baseline; orchestrator authorized archive (documented above under "Intentional Archive with Warnings").

## Specs Synced (main specs = source of truth)

| Domain | Action | Details |
|--------|--------|---------|
| `user-provisioning` | Verified — already consistent, **no change needed** | The delta (`openspec/changes/web-registration-provisioning/specs/user-provisioning/spec.md`) contains only `## ADDED Requirements` R1–R8 (17 scenarios) + Non-Goals — no MODIFIED/REMOVED/RENAMED. The main spec `openspec/specs/user-provisioning/spec.md` was created by the spec phase and already reflects the final state; a diff against the delta shows only header normalization (`## ADDED Requirements` → `## Purpose` + `## Requirements`, and "Out of scope for this change:" → "Out of scope for this capability:"). All requirement/scenario content is identical. |

The source of truth `openspec/specs/user-provisioning/spec.md` now reflects the new behavior: web registration accepts `academyName` + `confirmPassword`, provisions a per-user Turso DB (slug, 409-retry, token, 001–020 migrations), writes per-user + shared rows with a `user_databases` mapping, keeps payment lifecycle on the shared DB, cleans up on failure, and gates on Turso env vars.

## Verification Outcome

| Metric | Result |
|--------|--------|
| Tasks | 17/17 complete |
| Spec requirements | 8/8 compliant (R1–R8) |
| Scenarios | 17/17 compliant |
| CRITICAL findings | 0 |
| Focused suites | `provisioning.test.ts` 38/38 exit 0; `register-action.test.ts` 12/12 exit 0 |
| Full suite (web/) | 7 failed / 156 passed — exactly the pre-existing Stripe-era set (5× missing `lib/payments/stripe` imports, 2× `geoToGateway` expecting `'stripe'`) |
| Typecheck (web/) | 7 errors — exactly the pre-existing set (`db.ts` TS2352, `payments.test.ts` TS2307 ×5, `checkout-integration.test.ts` TS2367); the former `register.ts` TS2345 is gone |
| Build (web/) | PASS (`astro build` complete) |
| E2E (real Turso) | 9 passed / 8 skipped / 0 failed, ran green twice; register → per-user DB → dashboard "Panel de Control" with zero "Error de conexión" blocks |

## Open Follow-ups (documented, NOT blocking — schedule as new changes)

1. **FU1 — Backfill of existing web-registered users** (urgent): pre-change accounts have no per-user DB / `user_databases` row and remain broken until a backfill change lands. Non-Goal of this change; tracked from proposal/design/tasks.
2. **FU2 — Vercel env rollout**: set `TURSO_API_TOKEN` / `TURSO_ORG` / `TURSO_GROUP` in Vercel production **BEFORE** deploy. Documented in `web/docs/vercel-rollout.md`, NOT executed (`vercel` CLI/credentials unavailable). Registration fails closed by design (D6) until set — deploy order matters.
3. **FU3 — Stripe-era baseline cleanup (verify S4)**: 7 pre-existing failing tests + 7 pre-existing `tsc` errors (Stripe-era: missing `lib/payments/stripe` module imports, `geoToGateway('stripe')` expectations, `db.ts` TS2352, `checkout-integration` TS2367) — a separate cleanup change is needed to restore a green baseline (see also AGENTS.md guidance).
4. **FU4 (minor)**: E2E runs create real artifacts (per-user DBs + shared rows) not auto-cleaned — tag test rows/DBs to ease cleanup; monitor Vercel Hobby 10 s cold-start timeout on provisioning (design open question); 020_web_schema alignment (`plan_id` vs `plan`, grace cols, provider CHECKs) and `send-reminders` cron `s.grace_end` bug remain pre-existing Non-Goals.

## Rollback Notes

- Revert the 6 change commits (in order): `8e0e89b 189ec5f 38f0c9f b66e7d5 f21285c 6d209bd`.
- No shared-DB schema migration touched → shared DB needs no rollback.
- Residual per-user DBs from failed or test runs: remove via Turso API/CLI (`DELETE /v1/organizations/{org}/databases/{name}`).
- Vercel env vars (if set per FU2): `vercel env rm <VAR> production`.
- `auth.spec.ts` selector fix (`input[name="password"]` etc.) rolls back with `f21285c`; the two-password form requires it while the change is live.

## Engram Traceability (observation IDs — history preserved, not deleted)

| Artifact | Observation ID |
|----------|----------------|
| Exploration | #1108 |
| Proposal | #1109 |
| Spec (delta) | #1110 |
| Design | #1111 |
| Tasks | #1113 |
| Apply progress | #1114 |
| Verify report | #1116 |
| Archive report (this) | `sdd/web-registration-provisioning/archive-report` |

## Risks / Caveats Carried Forward

- Existing web-registered users remain broken until FU1 (backfill) lands.
- Registration fails closed in production until FU2 (Vercel vars) is done — intentional (D6), but a deploy without the vars breaks new signups with "Registro no disponible temporalmente".
- Full `web/` suite exits 1 on pre-existing Stripe-era failures, masking future regressions (FU3).
- `dbToken` inside the app JWT (7-day cookie) matches the desktop pattern; token rotation is a future consideration.
- Serverless cold-start latency on register (Vercel Hobby 10 s) — monitor; Fluid compute or queued provisioning (A2) later.
- `createDatabase` uses the server-reported `name`/`hostname` (W3) — consistent with desktop parity.
- 4-hex slug suffix = 16 bits of entropy; 409 retries absorb collisions (S2 — acceptable, no change needed).

## SDD Cycle Complete

Change `web-registration-provisioning` planned, implemented (`8e0e89b 189ec5f 38f0c9f b66e7d5 f21285c 6d209bd`), verified (8/8 requirements, 17/17 scenarios, 0 CRITICAL), and archived. Main specs already reflected the final delta state (no merge needed at archive); change folder moved to `openspec/changes/archive/2026-08-12-web-registration-provisioning/`. Ready for the next change.
