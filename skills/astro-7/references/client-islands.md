# Client Islands

Source: https://docs.astro.build/en/concepts/islands/

## Overview

Client islands are interactive JavaScript UI components that hydrate separately from the rest of the page. By default, Astro renders ALL UI components to static HTML with zero client-side JavaScript. You opt-in to interactivity per-component using `client:*` directives.

## Core Principle

```astro
<!-- Static: renders to HTML only, no JS shipped -->
<MyReactComponent />

<!-- Interactive island: hydrates on the client -->
<MyReactComponent client:load />
```

JavaScript is ONLY loaded for components marked with `client:*` directives. Everything else is pure HTML.

## Client Directives

### `client:load`

Hydrate immediately on page load. Use for components that need to be interactive right away (e.g., modals, interactive forms above the fold).

```astro
<InteractiveForm client:load />
```

### `client:idle`

Hydrate once the browser is idle (uses `requestIdleCallback`). Use for lower-priority interactive components that don't need immediate interactivity.

```astro
<Sidebar client:idle />
```

### `client:visible`

Hydrate when the component enters the viewport (uses `IntersectionObserver`). Best for below-the-fold components like carousels, comment sections, or charts.

```astro
<ImageCarousel client:visible />
<CommentsSection client:visible />
```

### `client:media={query}`

Hydrate when a CSS media query matches. Use for components only interactive at certain viewport sizes.

```astro
<MobileMenu client:media="(max-width: 768px)" />
```

### `client:only={framework}`

Skip server rendering entirely — render only on the client. Use for components that depend on browser-only APIs (canvas, WebGL, etc.).

```astro
<ThreeJSScene client:only="react" />
```

Note: you must specify the framework name as a string.

## Key Benefits

1. **Performance**: Only interactive components ship JavaScript
2. **Parallel loading**: Islands hydrate independently, no blocking
3. **Framework agnostic**: Mix React, Vue, Svelte, Solid on the same page
4. **Selective hydration**: Fine-grained control over when each component hydrates

## Sharing State Between Islands

Islands run in isolated component contexts, but can share state using [Nano Stores](https://github.com/nanostores/nanostores):

```typescript
// src/stores/counter.ts
import { atom } from 'nanostores';
export const count = atom(0);
```

```tsx
// React component A
import { useStore } from '@nanostores/react';
import { count } from '../stores/counter';

export function CounterA() {
  const $count = useStore(count);
  return <button onClick={() => count.set($count + 1)}>{$count}</button>;
}
```

```svelte
<!-- Svelte component B -->
<script>
import { count } from '../stores/counter';
</script>
<p>Count: {$count}</p>
```

Both components stay in sync, even across different frameworks.

## Supported Frameworks

- React / Preact
- Svelte
- Vue
- SolidJS
- HTMX
- Web Components (Lit, etc.)

Add integrations via:

```bash
npx astro add react
npx astro add svelte
npx astro add vue
```

## Best Practices

1. **Default to no directive** — most components don't need client JS
2. **Prefer `client:visible`** for anything below the fold
3. **Use `client:idle`** for non-critical interactive widgets
4. **Reserve `client:load`** for above-the-fold, immediately-interactive components
5. **Use `client:only`** sparingly — only when server rendering is impossible
6. **Keep islands small** — extract only the interactive part into a separate component

Content was rephrased for compliance with licensing restrictions.
