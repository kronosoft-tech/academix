# Actions

Source: https://docs.astro.build/en/guides/actions/

## Overview

Astro Actions provide type-safe backend functions callable from the client. They handle data fetching, JSON parsing, and Zod input validation automatically. Use actions instead of raw API endpoints for seamless client-server communication.

## Setup

Define actions in `src/actions/index.ts`:

```typescript
import { defineAction } from 'astro:actions';
import { z } from 'astro/zod';

export const server = {
  getGreeting: defineAction({
    input: z.object({
      name: z.string(),
    }),
    handler: async (input) => {
      return `Hello, ${input.name}!`;
    }
  })
};
```

## Calling Actions (Client-Side)

From a `<script>` tag or UI framework component:

```html
<script>
import { actions } from 'astro:actions';

const { data, error } = await actions.getGreeting({ name: "Houston" });
if (!error) alert(data);
</script>
```

### With `.orThrow()` (skip error handling)

```typescript
const greeting = await actions.getGreeting.orThrow({ name: "World" });
// Throws on error instead of returning { error }
```

## Organizing Actions

Group related actions using nested objects:

```typescript
// src/actions/user.ts
import { defineAction } from 'astro:actions';

export const user = {
  getUser: defineAction(/* ... */),
  createUser: defineAction(/* ... */),
};
```

```typescript
// src/actions/index.ts
import { user } from './user';

export const server = {
  myAction: defineAction({ /* ... */ }),
  user,
};
```

Call as `actions.user.getUser()`.

## Form Data

Accept form submissions with `accept: 'form'`:

```typescript
export const server = {
  newsletter: defineAction({
    accept: 'form',
    input: z.object({
      email: z.email(),
      terms: z.boolean(),
    }),
    handler: async ({ email, terms }) => { /* ... */ },
  })
};
```

### HTML Form Action (Zero-JS)

Submit forms without client-side JavaScript:

```astro
---
import { actions } from 'astro:actions';
---
<form method="POST" action={actions.logout}>
  <button>Log out</button>
</form>
```

### Get Action Result Server-Side

```astro
---
import { actions } from 'astro:actions';
const result = Astro.getActionResult(actions.createProduct);
if (result && !result.error) {
  return Astro.redirect(`/products/${result.data.id}`);
}
---
```

## Error Handling

### ActionError

```typescript
import { defineAction, ActionError } from "astro:actions";

export const server = {
  likePost: defineAction({
    input: z.object({ postId: z.string() }),
    handler: async (input, ctx) => {
      if (!ctx.cookies.has('user-session')) {
        throw new ActionError({
          code: "UNAUTHORIZED",
          message: "User must be logged in.",
        });
      }
      // ...
    },
  }),
};
```

### Input Validation Errors

```typescript
import { actions, isInputError } from 'astro:actions';

const { error } = await actions.newsletter(formData);
if (isInputError(error)) {
  if (error.fields.email) {
    const message = error.fields.email.join(', ');
  }
}
```

## Client Redirects

```tsx
import { actions } from 'astro:actions';
import { navigate } from 'astro:transitions/client';

export function LogoutButton() {
  return (
    <button onClick={async () => {
      const { error } = await actions.logout();
      if (!error) navigate('/');
    }}>
      Logout
    </button>
  );
}
```

## Calling Actions Server-Side

From Astro components or server endpoints:

```astro
---
import { actions } from 'astro:actions';
const { data, error } = await Astro.callAction(actions.findProduct, { query: "shoes" });
---
```

## Security

Actions are accessible as public endpoints at `/_actions/{action.name}`. Always add authorization checks in your handler:

```typescript
handler: async (_input, context) => {
  if (!context.locals.user) {
    throw new ActionError({ code: 'UNAUTHORIZED' });
  }
  return { /* data */ };
}
```

### Gate from Middleware

```typescript
import { getActionContext } from 'astro:actions';

export const onRequest = defineMiddleware(async (context, next) => {
  const { action } = getActionContext(context);
  if (action?.calledFrom === "rpc") {
    if (!context.cookies.has("user-session")) {
      return new Response("Forbidden", { status: 403 });
    }
  }
  return next();
});
```

## Return Data Format

Actions use the Devalue library for serialization — supports Dates, Maps, Sets, URLs beyond standard JSON. Inspect the `data` property from the action result for debugging (not the network response).

Content was rephrased for compliance with licensing restrictions.
