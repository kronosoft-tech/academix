# AGENTS.md - Backend (src-tauri/)

This file provides context for AI agents working on the Academix Tauri/Rust backend.

## Tech Stack

- **Tauri 2** (Rust)
- **Serde** for serialization
- **Vite** for frontend bundling

---

## Commands

```bash
# Full Tauri development (frontend + backend)
bun run tauri dev

# Build Tauri app
bun run tauri build

# Check Rust code (if cargo is available)
cargo check
```

---

## Rust Configuration

The backend uses:
- Rust edition 2021
- Tauri 2.x
- Serde with derive macros

Key files:
- `src-tauri/Cargo.toml` - Dependencies
- `src-tauri/src/lib.rs` - Command handlers
- `src-tauri/src/main.rs` - Application entry

---

## Tauri Commands

### Defining Commands

```rust
use tauri::command;

#[command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
```

### Registering Commands

In `lib.rs`, add to the `run()` function:

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Calling from Frontend

```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke<string>("greet", { name: "Alice" });
```

---

## Code Style

### Naming

- **Functions**: snake_case
- **Structs/Enums**: PascalCase
- **Modules**: snake_case
- **Constants**: SCREAMING_SNAKE_CASE

### Error Handling

```rust
use std::fs::read_to_string;

fn read_file(path: &str) -> Result<String, String> {
    read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))
}
```

### Async Commands

```rust
use tauri::command;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Response {
    data: String,
    status: u16,
}

#[command]
async fn async_command(name: String) -> Result<Response, String> {
    // Async logic here
    Ok(Response {
        data: format!("Hello, {}!", name),
        status: 200,
    })
}
```

---

## Dependencies

Add to `src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

---

## Skills

**MUST load these skills when working on:**

| Context | Skill | Command |
|---------|-------|---------|
| Tauri 2 | `tauri-2` | skill(name: "tauri-2") |
| Database | `tauri-sql` | skill(name: "tauri-sql") |

---

## Pre-commit Checklist

- [ ] Rust code compiles (`cargo check`)
- [ ] Frontend builds (`bun run build`)
- [ ] No panics in command handlers
- [ ] Proper error handling with Result types
- [ ] Serialization attributes on public structs
