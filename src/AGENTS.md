# AGENTS.md - Frontend (src/)

This file provides context for AI agents working on the Academix frontend.

## Tech Stack

- **React 19** with TypeScript
- **Vite** as bundler
- **Tauri 2** for desktop integration

---

## Commands

```bash
# Start development server
bun run dev

# Type check
bunx tsc --noEmit

# Build for production
bun run build
```

---

## TypeScript Configuration

Strict mode is enabled. Key settings from `tsconfig.json`:
- `strict: true`
- `noUnusedLocals: true`
- `noUnusedParameters: true`
- `noFallthroughCasesInSwitch: true`

---

## Code Style

### Import Order

```typescript
// 1. React core (if needed for JSX transform)
import React from "react";

// 2. React hooks
import { useState, useEffect } from "react";

// 3. External libraries
import { invoke } from "@tauri-apps/api/core";

// 4. Internal modules
import MyComponent from "./components/MyComponent";
import { useAuth } from "./hooks/useAuth";

// 5. Styles
import "./styles/main.css";
```

### File Naming

| Type | Convention | Example |
|------|------------|---------|
| Components | PascalCase | `UserProfile.tsx` |
| Hooks | camelCase | `useAuth.ts` |
| Utilities | camelCase | `formatDate.ts` |
| Types | PascalCase | `User.ts` |

### Component Pattern

```typescript
import { useState, useEffect } from "react";

interface Props {
  title: string;
  onSubmit?: (data: string) => void;
}

export function MyComponent({ title, onSubmit }: Props) {
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // Cleanup on unmount
    return () => {};
  }, []);

  const handleClick = () => {
    onSubmit?.("clicked");
  };

  return (
    <div>
      <h1>{title}</h1>
      <button onClick={handleClick}>Submit</button>
    </div>
  );
}
```

### Naming Conventions

- **Components**: PascalCase
- **Variables/functions**: camelCase
- **Constants**: UPPER_SNAKE_CASE
- **Interfaces**: PascalCase (no "I" prefix)

---

## Tauri Integration

### Calling Rust Commands

```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke<string>("command_name", { param: "value" });
```

### Error Handling

```typescript
try {
  const result = await invoke<string>("greet", { name });
} catch (error) {
  console.error("Command failed:", error);
}
```

---

## Testing

No test framework configured yet. When adding tests:

```bash
# Install Vitest
npm install -D vitest @testing-library/react @testing-library/jest-dom
```

---

## Context7 MCP

**ALWAYS use Context7** before writing code with external libraries:
- Run `context7_resolve-library-id` to get the library ID
- Then use `context7_query-docs` for API examples

Example for React 19:
```typescript
// 1. Resolve library
context7_resolve-library-id(libraryName: "react", query: "useState hook")
// 2. Query docs
context7_query-docs(libraryId: "/facebook/react", query: "useState examples")
```

---

## Skills

**MUST load these skills when working on:**

| Context | Skill | Command |
|---------|-------|---------|
| React 19 | `react-19` | skill(name: "react-19") |
| React + Tauri | `react-19-tauri` | skill(name: "react-19-tauri") |
| TypeScript | `typescript` | skill(name: "typescript") |
| Database | `tauri-sql` | skill(name: "tauri-sql") |
| State | `zustand-5` | skill(name: "zustand-5") |
| Styling | `tailwind-4` | skill(name: "tailwind-4") |

---

## Pre-commit Checklist

- [ ] `bunx tsc --noEmit` passes
- [ ] `bun run build` succeeds
- [ ] No `console.log` in production
- [ ] Use named exports for utilities/hooks
- [ ] Use default exports for page components
