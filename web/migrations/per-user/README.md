# Per-User Migrations (`web/migrations/per-user/`)

These are **copies** of the desktop migrations in `src-tauri/migrations/`.
They are applied to each newly provisioned per-user Turso database by
`runMigrationsOnDb()` in `web/src/lib/provisioning.ts` during web registration.

Do **not** hand-edit these files. Any schema change for per-user databases
belongs in `src-tauri/migrations/` first, then gets mirrored here.

## Sync Procedure (for future desktop `021+`)

1. Add the new migration as `src-tauri/migrations/021_*.sql` and register it in
   `src-tauri/src/lib.rs` (`run_local_migrations()` wiring) for the desktop app.
2. Copy it here byte-identical:

   ```bash
   cp "src-tauri/migrations/021_*.sql" web/migrations/per-user/
   ```

3. Verify the copy is byte-identical and sorted 001–020+:

   ```bash
   diff -r src-tauri/migrations/ web/migrations/per-user/ --exclude="*.md"
   ls web/migrations/per-user/ | sort
   ```

4. The web migration runner picks it up automatically (sorted by filename,
   guarded by `_schema_migrations`), so no code change is required in
   `provisioning.ts`.

Keep the full set of 20 files (001–020) present. A per-user database is only
migrated once at registration time, and `_schema_migrations` makes re-runs a
no-op, so later files are applied to every *new* database.

## `002_seed_admin.sql` — intentional no-op (design decision D3)

`002_seed_admin.sql` seeds `admin@academix.com` into the shared/local users
table. On a fresh per-user database the `INSERT OR IGNORE` swallows the NOT NULL
`id` violation, so it is a **silent no-op** for provisioning.

It is **kept** here (not excluded) because requirement R3 mandates copies of
001–020 and keeping the set byte-identical to `src-tauri/migrations/` keeps
`_schema_migrations` tracking identical between desktop and web. Do not "fix"
it — the file must stay as-is.
