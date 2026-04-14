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

## Architecture

```
academix/
├── src/                    # Frontend (React 19)
├── src-tauri/              # Backend (Rust/Tauri 2)
├── package.json            # npm scripts
├── vite.config.ts          # Vite config
├── tsconfig.json           # TypeScript config
└── AGENTS.md               # This file (command center)
```

---

## Pre-commit Checklist

Before any commit:
1. [ ] `bunx tsc --noEmit` passes
2. [ ] `bun run build` succeeds  
3. [ ] No `console.log` in production code
4. [ ] Types are explicit (avoid `any`)
