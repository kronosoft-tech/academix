# Tech Stack

## Frontend

- **React 19** with TypeScript (strict mode)
- **Vite 7** (dev server on port 1420, strict port)
- **Tailwind CSS v4** via `@tailwindcss/vite` plugin
- **Zustand v5** for state management
- **React Router v7** with `HashRouter` (required for Tauri `file://` protocol)
- **Lucide React** for icons
- **Chart.js / Recharts** for data visualization
- **jsPDF** for PDF generation
- **clsx + tailwind-merge** for class name utilities

## Backend

- **Tauri 2** (Rust)
- **SQLite** via `tauri-plugin-sql` (local) + **libSQL/Turso** (remote sync)
- **Hexagonal architecture**: domain → application → infrastructure → commands
- **bcrypt** for password hashing
- **chrono** for dates, **uuid** for IDs
- **async-trait** for async port definitions
- **thiserror** for error types

## Package Manager

**Bun** (not npm). All commands use `bun run`, `bunx`.

## Commands

```bash
# Install dependencies
bun install

# Type check (always before commit)
bunx tsc --noEmit

# Dev server (frontend only on :1420)
bun run dev

# Full Tauri dev (frontend + Rust backend)
bun run tauri dev

# Build frontend
bun run build

# Unit tests (Vitest + jsdom)
bun run test

# Unit tests in watch mode
bun run test:watch

# E2E tests (Playwright, needs dev server)
bun run test:e2e
```

## TypeScript Config

- Target: ES2020
- Strict mode enabled
- `noUnusedLocals: true`
- `noUnusedParameters: true`
- `noFallthroughCasesInSwitch: true`

## Testing

- **Unit**: Vitest + React Testing Library + jsdom. Files: `src/**/*.test.ts(x)`
- **E2E**: Playwright. Files: `tests/e2e/`
- Setup file: `src/test/setup.ts`

## Key Constraints

- Port 1420 is hardcoded and strict — dev fails if busy
- HashRouter required (BrowserRouter won't work under Tauri `file://`)
- Migrations are idempotent and run on every app startup
- No `console.log` in production code
