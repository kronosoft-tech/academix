# On-Demand Rendering (SSR)

Source: https://docs.astro.build/en/guides/on-demand-rendering/

## Overview

On-demand rendering generates pages per request on the server (SSR). This enables cookies, headers, personalized content, and real-time data without full-site rebuilds.

## Requirements

An **adapter** must be installed to enable SSR:

```bash
npx astro add node       # Node.js (Docker, self-hosted)
npx astro add netlify    # Netlify
npx astro add vercel     # Vercel
npx astro add cloudflare # Cloudflare
```

## Enabling On-Demand Rendering

### Per-Route (Default Static)

By default, all pages are prerendered (static). Opt individual routes into SSR:

```astro
---
export const prerender = false;
---
<html>
  <!-- This page renders on each request -->
</html>
```

### Server Mode (Default SSR)

For highly dynamic apps, render everything on-demand:

```javascript
// astro.config.mjs
import { defineConfig } from 'astro/config';
import node from '@astrojs/node';

export default defineConfig({
  output: 'server',
  adapter: node({ mode: 'standalone' }),
});
```

Then opt individual pages into prerendering:

```astro
---
export const prerender = true;
---
<!-- This page is built at build time -->
```

## SSR Features

### Cookies

```astro
---
export const prerender = false;
let counter = 0;
if (Astro.cookies.has('counter')) {
  const cookie = Astro.cookies.get('counter');
  counter = (cookie?.number() ?? 0) + 1;
}
Astro.cookies.set('counter', String(counter));
---
<h1>Counter = {counter}</h1>
```

### Request Object

```astro
---
export const prerender = false;
const cookie = Astro.request.headers.get('cookie');
const method = Astro.request.method; // GET, POST, etc.
const url = Astro.url; // Full request URL
---
```

### Response Headers

```astro
---
export const prerender = false;
Astro.response.headers.set('Cache-Control', 'public, max-age=3600');
Astro.response.status = 200;
---
```

### Return Response Object

```astro
---
export const prerender = false;
import { getProduct } from '../api';
const product = await getProduct(Astro.params.id);

if (!product) {
  return new Response(null, { status: 404, statusText: 'Not found' });
}

if (!product.isAvailable) {
  return Astro.redirect("/products", 301);
}
---
<h1>{product.name}</h1>
```

### HTML Streaming

Astro streams HTML in on-demand mode — components are sent to the browser as they render. No configuration needed.

## Server Endpoints (API Routes)

Create API endpoints in `src/pages/`:

```typescript
// src/pages/api/random.ts
export const prerender = false;

export async function GET() {
  return new Response(
    JSON.stringify({
      number: Math.random(),
      message: "Here's a random number",
    }),
  );
}
```

Supports all HTTP methods: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`.

## Official Adapters

| Adapter | Runtime | Best For |
|---------|---------|----------|
| `@astrojs/node` | Node.js | Docker, self-hosted, VPS |
| `@astrojs/netlify` | Netlify Edge/Functions | Netlify deployments |
| `@astrojs/vercel` | Vercel Serverless/Edge | Vercel deployments |
| `@astrojs/cloudflare` | Cloudflare Workers | Cloudflare Pages |

## When to Use SSR

- User authentication / personalized content
- Data that changes frequently
- Access to cookies and session management
- Real-time API data
- Dynamic redirects based on request
- Server islands (`server:defer`)

## When to Stay Static

- Content that rarely changes (blogs, docs)
- Marketing pages
- Pages without user-specific content
- Maximum performance (no server needed)

Content was rephrased for compliance with licensing restrictions.
