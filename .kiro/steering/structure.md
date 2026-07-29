# Project Structure

## Top Level

```
academix/
├── src/              # Frontend (React + TypeScript)
├── src-tauri/        # Backend (Rust + Tauri 2)
├── tests/            # E2E tests (Playwright)
├── public/           # Static assets
├── index.html        # Vite entry point
├── vite.config.ts
├── vitest.config.ts
├── tsconfig.json
└── package.json
```

## Frontend (`src/`)

Feature-based architecture:

```
src/
├── app/                    # App shell
│   ├── router.tsx          # Route definitions (HashRouter)
│   ├── layouts/            # Layout wrappers
│   ├── pages/              # Route-level page components
│   └── components/         # App-wide components (sidebar, nav)
├── features/               # Feature modules (self-contained)
│   ├── auth/
│   ├── students/
│   ├── courses/
│   ├── groups/
│   ├── attendance/
│   ├── payments/
│   ├── accounting/
│   ├── dashboard/
│   └── users/
├── shared/                 # Cross-feature shared code
│   ├── ui/components/      # Reusable UI components
│   ├── ui/icons/           # Icon components
│   ├── hooks/              # Shared hooks
│   ├── types/              # Shared type definitions
│   └── utils/              # Utility functions
├── lib/                    # Third-party integrations/wrappers
├── theme/                  # Theme/styling config
├── test/                   # Test setup and utilities
└── main.tsx                # App entry point
```

### Feature Module Convention

Each feature in `src/features/{name}/` follows:

```
feature/
├── components/     # Feature-specific components
├── hooks/          # Feature-specific hooks (IPC calls via invoke())
├── types/          # Feature-specific types
├── routes/         # Feature route components (if any)
└── index.ts        # Public API barrel export
```

## Backend (`src-tauri/src/`)

Hexagonal (ports & adapters) architecture:

```
src-tauri/src/
├── domain/              # Core business logic (no dependencies)
│   ├── entities/        # Domain entities
│   ├── value_objects/   # Value objects
│   └── errors.rs        # Domain errors
├── application/         # Use cases and ports
│   ├── use_cases/       # Application services
│   ├── ports/           # Port traits (interfaces)
│   ├── dto/             # Data transfer objects
│   └── errors.rs        # Application errors
├── infrastructure/      # External adapters
│   ├── repositories/    # Repository implementations
│   ├── turso/           # Turso/libSQL client
│   ├── local_db.rs      # Local SQLite setup
│   └── password.rs      # Bcrypt hashing
├── commands/            # Tauri IPC command handlers
│   ├── students.rs
│   ├── courses.rs
│   ├── auth.rs
│   └── ...
├── lib.rs               # Command registration (generate_handler![])
└── main.rs              # Tauri app entry
```

### Communication Flow

```
React component
  → invoke("command_name", { args })
    → Tauri command (commands/)
      → UseCase (application/use_cases/)
        → Port trait (application/ports/)
          → Repository impl (infrastructure/repositories/)
            → SQLite / Turso
```

## Database

- **Migrations**: `src-tauri/migrations/` (001–017, idempotent, run on startup)
- **Local**: SQLite via `tauri-plugin-sql`
- **Remote**: Turso (libSQL) for sync

## Tests

- Unit tests colocated: `src/**/*.test.ts(x)`
- E2E tests: `tests/e2e/`
