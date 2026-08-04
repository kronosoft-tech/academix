# Astro v7 Migration & Breaking Changes

Source: https://docs.astro.build/en/guides/upgrade-to/v7/

## Upgrade Command

```bash
npx @astrojs/upgrade   # npm
pnpm dlx @astrojs/upgrade  # pnpm
yarn dlx @astrojs/upgrade   # yarn
```

## Breaking Changes

### 1. Vite 8

Astro v7 uses Vite 8. Check the [Vite 8 migration guide](https://vite.dev/guide/migration) for plugin/config changes. Most projects won't need changes.

### 2. Rust Compiler (Replaces Go Compiler)

The Rust compiler is now the default and ONLY compiler. It is faster but stricter:

**Unclosed tags now error:**

```astro
<!-- BREAKS in v7 -->
<p>Hello world

<!-- CORRECT -->
<p>Hello world</p>
```

**Invalid HTML nesting is not auto-corrected:**

```astro
<!-- BREAKS layout in v7 (browser closes <p> early) -->
<p>
  <div>Content</div>
</p>

<!-- CORRECT -->
<div>
  <div>Content</div>
</div>
```

**CSS differences (cosmetic only):**
- Named colors may become hex (`rebeccapurple` → `#639`)
- `url()` values may gain/lose quotes

### 3. Reserved File: `src/fetch.ts`

`src/fetch.ts` is now used for advanced routing. If you have one for other purposes:

```javascript
// astro.config.mjs
export default defineConfig({
  fetchFile: './src/router.ts',  // Use different file
  // fetchFile: null,            // Or disable advanced routing
});
```

### 4. Sätteri Markdown Processor

Sätteri replaces remark/rehype as the default Markdown processor.

**No remark/rehype plugins?** No action needed.

**Using remark/rehype plugins?** Install `@astrojs/markdown-remark` and switch:

```bash
npm install @astrojs/markdown-remark
```

```javascript
import { defineConfig } from 'astro/config';
import { unified } from '@astrojs/markdown-remark';

export default defineConfig({
  markdown: {
    processor: unified(),
  },
});
```

### 5. Default Whitespace: `compressHTML: 'jsx'`

JSX-style whitespace stripping is now default. Adjacent inline elements lose spaces:

```astro
<!-- Renders as "helloworld" in v7 (was "hello world" in v6) -->
<span>hello</span>
<em>world</em>

<!-- Fix: explicit space -->
<span>hello</span> <em>world</em>
```

Revert to old behavior:

```javascript
export default defineConfig({
  compressHTML: true, // HTML-aware compression (v6 behavior)
});
```

### 6. `@astrojs/db` Removed

Replace with:
- **Node.js built-in SQLite** (`node:sqlite`) — for Node adapter
- **Drizzle ORM** — if using Drizzle-based schema/query API
- **Turso / Neon / PlanetScale** — for hosted databases

### 7. View Transitions Internals Removed

These APIs no longer exist:
- `TRANSITION_BEFORE_PREPARATION` → use string `'astro:before-preparation'`
- `TRANSITION_AFTER_PREPARATION` → use string `'astro:after-preparation'`
- `TRANSITION_BEFORE_SWAP` → use string `'astro:before-swap'`
- `TRANSITION_AFTER_SWAP` → use string `'astro:after-swap'`
- `TRANSITION_PAGE_LOAD` → use string `'astro:page-load'`
- `isTransitionBeforePreparationEvent()` → check `event.type === 'astro:before-preparation'`
- `createAnimationScope()` → removed entirely

### 8. Container Renderer Import Path

```typescript
// OLD (deprecated)
import { getContainerRenderer } from '@astrojs/react';

// NEW
import { getContainerRenderer } from '@astrojs/react/container-renderer';
```

## Experimental Flags Now Stable

Remove these from `experimental` block — they're now top-level or default:

| Flag | Status in v7 |
|------|-------------|
| `logger` | Stable — use top-level `logger` config |
| `queuedRendering` | Default behavior (no config needed) |
| `rustCompiler` | Default and only compiler |
| `advancedRouting` | Enabled by default |
| `cache` | Stable — use top-level `cache` and `routeRules` |
| `routeRules` | Stable — use top-level `routeRules` |

### Updated Config

```javascript
import { defineConfig, logHandlers, memoryCache } from 'astro/config';

export default defineConfig({
  // These were experimental, now top-level:
  logger: logHandlers.json({ pretty: true }),
  cache: {
    provider: memoryCache(),
  },
  routeRules: {
    '/blog/[...path]': { maxAge: 300, swr: 60 },
  },
});
```

## Quick Checklist

1. [ ] Run `npx @astrojs/upgrade`
2. [ ] Remove experimental flags that are now stable
3. [ ] Fix unclosed HTML tags (run build to find them)
4. [ ] Check for invalid HTML nesting (`<div>` inside `<p>`, etc.)
5. [ ] Add explicit spaces between inline elements if needed
6. [ ] Rename `src/fetch.ts` if exists (or set `fetchFile`)
7. [ ] Install `@astrojs/markdown-remark` if using remark/rehype plugins
8. [ ] Replace `@astrojs/db` with alternative
9. [ ] Update container renderer imports
10. [ ] Check Vite 8 compatibility for custom plugins

Content was rephrased for compliance with licensing restrictions.
