# Proposal: Animate Interface - Fix anime.js v4 ESM Integration

## Intent

El proyecto Academix tiene funciones de animación placeholder en `src/features/accounting/lib/animations.ts` debido a problemas de ESM con anime.js v4. El usuario ha solicitado **mantener anime.js v4** y arreglar los problemas de importación ESM en lugar de downgradear a v3.

Este cambio busca:
1. Implementar las funciones de animación placeholder usando anime.js v4
2. Crear hooks de React para facilitar el uso de animaciones en componentes
3. Refactorizar Modal y MainLayout para integrar animaciones

## Scope

### In Scope
- **Fix anime.js v4 ESM import**: Resolver problemas de importación ESM en Vite/React 19
- **Implement animations.ts**: Completar todas las funciones placeholder con implementaciones reales usando anime.js v4
- **Create React animation hooks**: Crear hooks reutilizables (`useAnimation`, `useFadeIn`, etc.)
- **Refactor Modal component**: Integrar animaciones de entrada/salida en el componente Modal
- **Refactor MainLayout**: Añadir animaciones de transición en el layout principal
- **Test animations**: Verificar que las animaciones funcionan correctamente en desarrollo y producción

### Out of Scope
- Downgrade a anime.js v3 (EXPLÍCITAMENTE DESCARTADO - el usuario lo negó)
- Animaciones complejas de scroll (ScrollTrigger) - queda para futuro
- Animaciones 3D o canvas - fuera del scope actual

## Approach

### Paso 1: Diagnosticar y Fix ESM Import
Investigar el problema específico de ESM con anime.js v4 en el entorno Vite + React 19:
- Verificar si es un problema de export default vs named exports
- Probar imports usando `import * as anime from 'animejs'` o dynamic imports
- Configurar vite.config.ts si es necesario para manejo de ESM
- Si falla, crear wrapper que use `import('animejs')` de forma lazy

### Paso 2: Implementar animations.ts
Completar las funciones placeholder con implementaciones reales:
- fadeInCards, fadeOut, slideInFromLeft/Right, scaleIn/Out
- countUp, animateTableRows, bounce, pulse, animateProgressBar, shake, layoutShift
- Usar `anime.default()` o `anime()` dependiendo del export que funcione

### Paso 3: Crear React Animation Hooks
Crear hooks en `src/hooks/`:
- `useAnimation`: hook genérico para cualquier animación
- `useFadeIn`, `useFadeOut`, `useSlideIn`, etc.: hooks específicos
- Soporte para cleanup en unmount usando `anime.remove()`

### Paso 4: Integrar en Componentes
- Modal: animaciones de scale + fade en open/close
- MainLayout: animaciones de página/transición de rutas

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `package.json` | No Change | Mantener animejs ^4.3.6 (SIN CAMBIOS) |
| `src/features/accounting/lib/animations.ts` | Modified | Implementar todas las funciones placeholder |
| `src/hooks/` | New | Crear animation hooks |
| `src/features/accounting/components/Modal.tsx` | Modified | Integrar animaciones |
| `src/app/layouts/MainLayout.tsx` | Modified | Integrar transiciones |
| `vite.config.ts` | Modified (if needed) | Fix ESM config for anime.js v4 |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| ESM issues persist despite fixes | Medium | Usar dynamic import como fallback, o crear wrapper con lazy loading |
| Animations no funcionan en producción | Low | Testear con `bun run build` y verificar output |
| Hooks causan memory leaks | Low | Implementar cleanup proper con anime.remove() |
| Conflict with React strict mode | Low | Usar refs y cleanup en useEffect return |

## Rollback Plan

1. **Si ESM no se puede arreglar**: mantener animations.ts como placeholders, crear archivo `.disable` 说明 motivo
2. **Si hook tiene leaks**: agregar `anime.removeAll()` en cleanup
3. **Si break producción**: revertir a versión anterior de animations.ts (placeholders)

Rollback específico:
- `git checkout HEAD~1 -- src/features/accounting/lib/animations.ts`
- Eliminar hooks creados si fallan

## Dependencies

- **anime.js v4.3.6**: Ya en package.json, mantenerlo
- **React 19**: Entorno actual del proyecto
- **Vite**: Bundler actual

## Success Criteria

- [ ] `anime` se importa correctamente en animations.ts (no hay errores de ESM)
- [ ] Todas las funciones en animations.ts tienen implementación real (no placeholders)
- [ ] Las animaciones funcionan en `bun run dev` y `bun run build`
- [ ] Hooks de animación se pueden usar en componentes sin memory leaks
- [ ] Modal tiene animaciones de entrada y salida
- [ ] MainLayout tiene al menos una transición animada
- [ ] `bunx tsc --noEmit` pasa sin errores