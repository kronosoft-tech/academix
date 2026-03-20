# Proposal: Add SQLite Persistence

## Intent

Reemplazar los repositorios in-memory (HashMap + RwLock) con repositorios SQLite usando `tauri-plugin-sql`. Actualmente los datos se pierden al cerrar la app. Este cambio garantiza persistencia de datos.

## Scope

### In Scope
- Configurar `tauri-plugin-sql` con SQLite
- Definir migrations para: users, students, courses, groups, payments, attendance
- Implementar `SqliteUserRepository`, `SqliteStudentRepository`, etc.
- Reemplazar `InMemory*Repository` con `Sqlite*Repository` en lib.rs
- Actualizar capabilities para permisos SQL

### Out of Scope
- Migración de datos existentes (no hay datos persistidos)
- Tests de integración con base de datos
- Migraciones de esquema post-deployment

## Approach

1. Agregar `tauri-plugin-sql` con feature `sqlite`
2. Definir migrations en Rust con `MigrationKind::Up`
3. Crear repositorios SQLite que implementen las traits existentes (`UserRepository`, `StudentRepository`, etc.)
4. Actualizar `lib.rs` para usar `Sqlite*Repository` en lugar de `InMemory*Repository`
5. Agregar permissions en `capabilities/default.json`

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src-tauri/Cargo.toml` | Modified | Agregar `tauri-plugin-sql` con feature sqlite |
| `src-tauri/src/lib.rs` | Modified | Usar Sqlite repositorios, registrar migrations |
| `src-tauri/src/infrastructure/database/` | New + Modified | Definir migrations, DbPool wrapper |
| `src-tauri/src/infrastructure/repositories/*.rs` | Modified | Cambiar InMemory*Repository por Sqlite*Repository |
| `src-tauri/capabilities/default.json` | Modified | Agregar permisos sql:default |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Breaking changes en API de repositorios | Low | Mantener las traits existentes, solo cambiar implementación |
| Performance degradado por I/O | Low | SQLite es rápido para app de escritorio |
| Errores de migración | Low | Tests con migrations en dev primero |

## Rollback Plan

1. Revertir cambios en lib.rs para usar InMemory*Repository
2. Mantener migrations para no perder schema en DB existente
3. Eliminar feature sqlite de Cargo.toml si hay problemas

## Dependencies

- `tauri-plugin-sql` crate
- Frontend: `@tauri-apps/plugin-sql` npm package

## Success Criteria

- [ ] App inicia sin errores con SQLite
- [ ] Crear estudiante → cerrar app → abrir app → estudiante persiste
- [ ] Todos los CRUDs funcionan (students, courses, groups, payments, attendance, users)
- [ ] Tests pasan: `cargo test`
- [ ] Build exitoso: `bun run tauri build`
