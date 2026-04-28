# AGENTS.md - Academix (Command Center)

This is the **main command center** for AI agents working on Academix.

## Project Overview

Academix is a **Tauri 2 + React 19 + Vite + TypeScript** desktop application:
- **Frontend**: React 19 + TypeScript in `src/`
- **Backend**: Tauri 2 (Rust) in `src-tauri/`

---

## Quick Reference

| Component | Location | Specific AGENTS.md |
|-----------|----------|-------------------|
| Frontend | `src/` | [src/AGENTS.md](src/AGENTS.md) |
| Backend | `src-tauri/` | [src-tauri/AGENTS.md](src-tauri/AGENTS.md) |

---

## Global Commands

```bash
# Install all dependencies
bun install

# Type check (always run before commit)
bunx tsc --noEmit

# Build frontend
bun run build

# Full Tauri development
bun run tauri dev

# Build Tauri app
bun run tauri build

# Run tests
bun run test
bun run test:e2e
```

---

## Project Structure

```
academix/
├── src/                    # Frontend (React 19)
│   ├── app/               # Layout, router, componentes globales
│   │   ├── components/    # Shared UI components
│   │   ├── layouts/       # Layout components
│   │   ├── pages/         # Page components
│   │   └── router.tsx     # Application routing
│   ├── features/          # Módulos por funcionalidad
│   │   ├── auth/          # Autenticación
│   │   ├── dashboard/     # Dashboard principal
│   │   ├── students/      # Gestión de estudiantes
│   │   ├── courses/       # Gestión de cursos
│   │   ├── groups/        # Gestión de grupos
│   │   ├── attendance/    # Registro de asistencia
│   │   ├── payments/      # Seguimiento de pagos
│   │   ├── accounting/     # Módulo contable
│   │   └── users/         # Administración de usuarios
│   ├── shared/            # Componentes, hooks, tipos compartidos
│   │   ├── components/    # UI components reutilizables
│   │   ├── hooks/         # Hooks reutilizables
│   │   ├── types/          # Tipos TypeScript
│   │   └── utils/         # Utilidades
│   ├── theme/             # Theme and styling utilities
│   └── lib/               # Configuración y utilidades
├── src-tauri/              # Backend (Rust/Tauri 2)
│   ├── domain/            # Entidades y lógica de dominio
│   │   ├── entities/       # Domain entities
│   │   └── value_objects/ # Value objects
│   ├── application/       # Casos de uso, DTOs y puertos
│   │   ├── dtos/          # Data Transfer Objects
│   │   ├── ports/         # Port interfaces
│   │   └── use_cases/     # Application use cases
│   ├── infrastructure/   # Repositorios y base de datos
│   │   └── repositories/   # Repository implementations
│   ├── commands/          # Comandos Tauri
│   │   ├── accounting/     # Accounting commands
│   │   ├── attendance/    # Attendance commands
│   │   ├── auth/          # Authentication commands
│   │   ├── base/          # Base commands
│   │   ├── courses/       # Course commands
│   │   ├── employees/     # Employee commands
│   │   ├── groups/        # Group commands
│   │   ├── invoices/      # Invoice commands
│   │   ├── payments/      # Payment commands
│   │   ├── payroll/       # Payroll commands
│   │   ├── pdf/           # PDF generation commands
│   │   ├── register/      # Registration commands
│   │   └── users/         # User commands
│   ├── migrations/         # Database migrations
│   └── src/
│       ├── lib.rs         # Library entry point
│       └── main.rs        # Main application entry
├── tests/                  # Test files
├── package.json            # npm scripts
├── vite.config.ts         # Vite config
├── tsconfig.json           # TypeScript config
└── AGENTS.md              # This file (command center)
```

---

## Context7 MCP - Required for Documentation

**MUST use Context7** when:
- Asking about any library/framework API
- Needing code examples for external packages
- Working with React, Tauri, Vite, or any dependency

```typescript
// Example: Get React 19 documentation
// 1. First resolve library: context7_resolve-library-id
// 2. Then query: context7_query-docs
```

Always search Context7 before writing code that uses external APIs. This ensures you have up-to-date information and working examples.

---

## Skills to Use

Load these skills when working on specific areas:

| Context | Skill | When to use |
|---------|-------|-------------|
| React 19 | `react-19` | React components, hooks, state |
| React + Tauri | `react-19-tauri` | Frontend for Tauri desktop apps |
| TypeScript | `typescript` | Types, interfaces, generics |
| Tauri 2 | `tauri-2` | Rust commands, IPC, plugins |
| Tauri SQL | `tauri-sql` | SQLite, databases |
| Testing | `playwright` | E2E tests |
| State | `zustand-5` | If adding global state |
| Styling | `tailwind-4` | If adding Tailwind |
| Animations | `animejs` | Anime.js v4 animations |

---

## Development Workflow

### Starting Development
1. Run `bun install` to install dependencies
2. Use `bun run tauri dev` to start full development environment
3. Frontend runs on http://localhost:1420

### Adding New Features
1. Follow the existing feature structure in `src/features/`
2. Create new modules with appropriate subdirectories
3. Use Tauri commands for backend operations
4. Implement proper error handling on both frontend and backend

### Testing
- Unit tests: `bun run test`
- E2E tests: `bun run test:e2e`
- Test files located in `tests/` directory

---

## Architecture Guidelines

### Frontend (React 19)
- Strict TypeScript mode enabled
- Screaming architecture with feature-based organization
- Zustand for state management
- Tailwind CSS for styling
- React Router for navigation

### Backend (Tauri/Rust)
- Hexagonal architecture pattern
- Domain-driven design principles
- SQLite database with Tauri SQL plugin
- Command-based API for frontend integration

### Communication
- Frontend calls backend via `invoke()` from `@tauri-apps/api/core`
- Backend returns structured responses with proper error handling
- Async operations properly handled with Rust `Result` types

---

## Pre-commit Checklist

Before any commit:
1. [ ] `bunx tsc --noEmit` passes
2. [ ] `bun run build` succeeds  
3. [ ] No `console.log` in production code
4. [ ] Types are explicit (avoid `any`)
5. [ ] Run `bun run test` to verify unit tests pass
6. [ ] Run `bun run test:e2e` for E2E tests if relevant