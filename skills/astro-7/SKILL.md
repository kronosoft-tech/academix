# Astro 7 Skill

## Trigger

Working with Astro framework, building web pages in the `/web` directory, Astro components, layouts, pages, routing, SSR, islands architecture, content collections, actions, view transitions, or middleware.

## Context

Astro 7 is a JavaScript web framework optimized for building fast, content-driven websites. It uses an Islands Architecture for optimal performance — rendering pages to static HTML by default and only hydrating interactive components when needed.

**Version**: Astro 7.1 (current stable as of July 2026)
**Bundler**: Vite 8
**Compiler**: Rust-based (replaced Go compiler in v7)
**Markdown**: Sätteri processor (replaced remark/rehype as default)

## Project Integration

This skill supports the `/web` directory in the Academix monorepo — the public-facing marketing/landing site built with Astro 7.

## Core Concepts

### Islands Architecture

Astro renders pages to static HTML by default. Interactive components ("islands") are hydrated separately:

- **Client Islands**: Interactive JavaScript components hydrated on the client using `client:*` directives
- **Server Islands**: Dynamic server-rendered components loaded independently using `server:defer` directive

### Rendering Modes

- **Static (default)**: Pages prerendered at build time
- **On-demand (SSR)**: Pages rendered per-request with an adapter installed
- **Hybrid**: Mix static + on-demand per route using `export const prerender = false|true`
- **Server mode**: `output: 'server'` renders all pages on-demand by default

### Key Features (v7)

| Feature | Reference |
|---------|-----------|
| Server Islands | `references/server-islands.md` |
| Client Islands | `references/client-islands.md` |
| Actions (type-safe RPC) | `references/actions.md` |
| View Transitions | `references/view-transitions.md` |
| Content Collections | `references/content-collections.md` |
| On-demand Rendering (SSR) | `references/on-demand-rendering.md` |
| Middleware | `references/middleware.md` |
| v7 Migration/Breaking Changes | `references/v7-migration.md` |

## Quick Reference

### Project Structure

```
web/
├── astro.config.mjs        # Astro configuration
├── src/
│   ├── pages/              # File-based routing (.astro, .md, .mdx)
│   ├── layouts/            # Layout components
│   ├── components/         # Reusable components
│   ├── content/            # Content collection entries (markdown, etc.)
│   ├── content.config.ts   # Build-time collection definitions
│   ├── live.config.ts      # Live collection definitions (optional)
│   ├── actions/            # Server actions (index.ts)
│   ├── middleware.ts       # Request/response middleware
│   └── styles/             # Global styles
├── public/                 # Static assets (served as-is)
└── package.json
```

### Component Syntax

```astro
---
// Frontmatter: server-side JavaScript (runs at build/request time)
import Layout from '../layouts/Layout.astro';
import Counter from '../components/Counter.tsx';

const title = "My Page";
const data = await fetch('https://api.example.com/data').then(r => r.json());
---

<Layout title={title}>
  <!-- Static HTML by default -->
  <h1>{title}</h1>

  <!-- Client Island: hydrated on the client -->
  <Counter client:load initialCount={0} />

  <!-- Server Island: rendered on-demand server-side -->
  <UserAvatar server:defer>
    <div slot="fallback">Loading...</div>
  </UserAvatar>
</Layout>

<style>
  /* Scoped CSS by default */
  h1 { color: navy; }
</style>
```

### Client Directives

| Directive | Behavior |
|-----------|----------|
| `client:load` | Hydrate immediately on page load |
| `client:idle` | Hydrate once browser is idle |
| `client:visible` | Hydrate when element enters viewport |
| `client:media={query}` | Hydrate when media query matches |
| `client:only={framework}` | Skip server render, client-only |

### Server Directives

| Directive | Behavior |
|-----------|----------|
| `server:defer` | Turn component into a server island (renders on-demand) |

### Supported UI Frameworks

React, Preact, Svelte, Vue, SolidJS, HTMX, web components — can mix multiple on the same page.

## v7 Breaking Changes Summary

1. **Vite 8** — check Vite 8 migration guide for plugin compatibility
2. **Rust compiler** — stricter HTML validation, unclosed tags now error
3. **Sätteri Markdown** — new default processor (remark/rehype available via `@astrojs/markdown-remark`)
4. **`compressHTML: 'jsx'`** — new default whitespace handling (JSX rules)
5. **`src/fetch.ts` reserved** — used for advanced routing; rename existing files
6. **`@astrojs/db` removed** — use Drizzle, node:sqlite, or Turso instead
7. **Experimental flags stabilized**: `rustCompiler`, `queuedRendering`, `advancedRouting`, `cache`, `routeRules`, `logger`

## Commands

```bash
# Create new Astro project
npm create astro@latest

# Development
npx astro dev

# Build
npx astro build

# Preview production build
npx astro preview

# Add integration
npx astro add react
npx astro add tailwind
npx astro add node  # adapter for SSR

# Generate encryption key for server islands
npx astro create-key
```

## Best Practices

1. **Default to static** — only opt into SSR (`prerender = false`) when you need cookies, headers, or real-time data
2. **Use server islands** for personalized content (user avatars, auth-dependent UI) on otherwise static pages
3. **Use client islands sparingly** — only add `client:*` directives when JavaScript interactivity is truly needed
4. **Prefer `client:visible`** for below-the-fold interactive components to improve initial page load
5. **Use content collections** for structured content — get type safety, validation, and optimized querying
6. **Use Actions** instead of raw API endpoints for type-safe client-server communication
7. **Close all HTML tags** — the Rust compiler is strict about this (no auto-correction)
8. **Add explicit spaces** between inline elements: `<span>hello</span>{" "}<em>world</em>`

## Source

Content synthesized from official Astro documentation at https://docs.astro.build (Astro v7.0.2, Starlight v0.41.0). Content was rephrased for compliance with licensing restrictions.
