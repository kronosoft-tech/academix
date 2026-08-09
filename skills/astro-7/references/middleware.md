# Middleware

Source: https://docs.astro.build/en/guides/middleware/

## Overview

Middleware intercepts requests and responses, injecting behavior before pages/endpoints render. It runs at build time for prerendered pages and at request time for on-demand pages. Use it for auth, logging, header manipulation, and sharing data across routes via `locals`.

## Basic Usage

Create `src/middleware.ts` (or `src/middleware/index.ts`):

```typescript
import { defineMiddleware } from "astro:middleware";

export const onRequest = defineMiddleware((context, next) => {
  // Set data accessible in all pages/endpoints
  context.locals.user = { id: 1, name: "John" };
  context.locals.startTime = Date.now();

  return next();
});
```

Access in any `.astro` file:

```astro
---
const { user } = Astro.locals;
---
<h1>Welcome, {user.name}</h1>
```

## The Context Object

Properties available on `context`:

- `context.locals` — shared data object (lives per-request)
- `context.cookies` — cookie access
- `context.request` — the Request object
- `context.url` — the URL
- `context.redirect()` — send a redirect
- `context.rewrite()` — render a different page
- `context.isPrerendered` — whether this is a prerendered route

## Type Safety

Extend `App.Locals` for autocompletion in `env.d.ts`:

```typescript
declare namespace App {
  interface Locals {
    user: { id: number; name: string } | null;
    session: import("./lib/session").Session | null;
  }
}
```

## Chaining Middleware

Use `sequence()` to compose multiple middlewares:

```typescript
import { sequence } from "astro:middleware";

async function auth(context, next) {
  const session = context.cookies.get('session');
  context.locals.user = session ? await getUser(session.value) : null;
  return next();
}

async function logging(context, next) {
  console.log(`${context.request.method} ${context.url.pathname}`);
  const response = await next();
  console.log(`Response: ${response.status}`);
  return response;
}

export const onRequest = sequence(auth, logging);
```

Execution order: request flows through in order, response flows back in reverse.

## Response Modification

Intercept and transform the response:

```typescript
export const onRequest = async (context, next) => {
  const response = await next();
  const html = await response.text();
  const modified = html.replaceAll("PRIVATE INFO", "REDACTED");
  return new Response(modified, {
    status: 200,
    headers: response.headers
  });
};
```

## Rewrites

Render a different page without redirecting the user:

```typescript
import { isLoggedIn } from "~/auth.js";

export function onRequest(context, next) {
  if (!isLoggedIn(context)) {
    return context.rewrite(new Request("/login", {
      headers: { "x-redirect-to": context.url.pathname }
    }));
  }
  return next();
}
```

Or rewrite without re-triggering middleware:

```typescript
export function onRequest(context, next) {
  if (!isLoggedIn(context)) {
    return next("/login"); // Rewrite in place, no middleware re-run
  }
  return next();
}
```

## Common Patterns

### Authentication Guard

```typescript
export const onRequest = defineMiddleware(async (context, next) => {
  if (context.isPrerendered) return next();

  const session = context.cookies.get('session-token');
  if (!session && context.url.pathname.startsWith('/dashboard')) {
    return context.redirect('/login');
  }

  if (session) {
    context.locals.user = await validateSession(session.value);
  }

  return next();
});
```

### CORS Headers

```typescript
export const onRequest = defineMiddleware(async (context, next) => {
  const response = await next();
  response.headers.set('Access-Control-Allow-Origin', '*');
  return response;
});
```

### Performance Timing

```typescript
export const onRequest = defineMiddleware(async (context, next) => {
  const start = Date.now();
  const response = await next();
  const duration = Date.now() - start;
  response.headers.set('X-Response-Time', `${duration}ms`);
  return response;
});
```

## Error Pages

- Middleware runs for all on-demand routes, including 404 pages
- If middleware itself errors, `Astro.locals` won't be available in the 500 page
- Skip prerendered routes with `if (context.isPrerendered) return next()`

## Key Gotcha

`locals` lives and dies within a single request. Store persistent data elsewhere (cookies, DB, session store).

Content was rephrased for compliance with licensing restrictions.
