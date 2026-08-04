# View Transitions

Source: https://docs.astro.build/en/guides/view-transitions/

## Overview

Astro provides animated transitions between pages using the View Transitions browser API. The `<ClientRouter />` component enables client-side routing (SPA mode) with smooth animations, persistent elements, and custom navigation behavior.

## Two Approaches

1. **Browser-native cross-document view transitions** — simple CSS animations between MPA pages, no extra JS
2. **`<ClientRouter />`** — Astro's built-in SPA router with enhanced features, fallback support, and lifecycle hooks

## Enabling SPA Mode

Import and add `<ClientRouter />` to your `<head>` or shared layout:

```astro
---
import { ClientRouter } from "astro:transitions";
---
<html>
  <head>
    <ClientRouter />
  </head>
  <body>
    <slot />
  </body>
</html>
```

## Transition Directives

### `transition:name`

Override automatic element matching for animations:

```astro
<aside transition:name="hero">
```

Must be unique per page.

### `transition:animate`

Set animation type:

| Value | Behavior |
|-------|----------|
| `fade` (default) | Crossfade between old and new |
| `slide` | Old slides out left, new slides in right |
| `initial` | Use browser default styling |
| `none` | Disable animations |

```astro
<main transition:animate="slide">
```

### `transition:persist`

Keep elements/components alive across navigations (preserves state):

```astro
<video controls muted autoplay transition:persist>
  <source src="/video.mp4" type="video/mp4" />
</video>

<!-- Preserve framework component state -->
<Counter client:load transition:persist initialCount={5} />
```

### `transition:persist-props`

Keep existing props instead of re-rendering with new page's props:

```astro
<Counter client:load transition:persist transition:persist-props />
```

## Custom Animations

```astro
---
import { fade } from "astro:transitions";
---
<header transition:animate={fade({ duration: "0.4s" })}>
```

Define fully custom animations:

```astro
---
const customTransition = {
  forwards: {
    old: { name: "slideOut", duration: "0.3s", easing: "ease-in" },
    new: { name: "slideIn", duration: "0.3s", easing: "ease-out" },
  },
  backwards: {
    old: { name: "slideIn", duration: "0.3s", direction: "reverse" },
    new: { name: "slideOut", duration: "0.3s", direction: "reverse" },
  },
};
---
<main transition:animate={customTransition}>
```

## Router Control

### Prevent Client Navigation

Force full-page reload on specific links:

```html
<a href="/page" data-astro-reload>Full reload</a>
```

### Programmatic Navigation

```typescript
import { navigate } from "astro:transitions/client";
navigate("/new-page");
navigate("/page", { history: "replace" }); // Don't add to history
```

### History Control

```html
<a href="/main" data-astro-history="replace">
```

Values: `push` | `replace` | `auto` (default)

### Form Transitions

Forms trigger transitions automatically with `<ClientRouter />`:

```html
<form action="/contact" method="POST" enctype="application/x-www-form-urlencoded">
```

Opt out with `data-astro-reload`:

```html
<form action="/contact" data-astro-reload>
```

## Lifecycle Events

In order of execution during navigation:

| Event | Phase | Use Case |
|-------|-------|----------|
| `astro:before-preparation` | Before content loads | Show loading spinner |
| `astro:after-preparation` | Content loaded | Hide loading spinner |
| `astro:before-swap` | Before DOM swap | Modify new document before render |
| `astro:after-swap` | After DOM swap | Set dark mode class, restore state |
| `astro:page-load` | Navigation complete | Set up event listeners, initialize scripts |

### Common Pattern: Re-initialize Scripts

```html
<script>
document.addEventListener("astro:page-load", () => {
  // Runs on every navigation, including initial page load
  document.querySelector(".hamburger")?.addEventListener("click", () => {
    document.querySelector(".nav-links")?.classList.toggle("expanded");
  });
});
</script>
```

### Dark Mode Preservation

```html
<script is:inline>
function applyTheme() {
  localStorage.theme === "dark"
    ? document.documentElement.classList.add("dark")
    : document.documentElement.classList.remove("dark");
}
document.addEventListener("astro:after-swap", applyTheme);
applyTheme();
</script>
```

## Script Behavior

- **Bundled module scripts**: Execute once, never re-run after navigation
- **Inline scripts**: May re-execute on navigation
- **`data-astro-rerun`**: Force inline script re-execution on every transition

```html
<script is:inline data-astro-rerun>
  // Re-runs on every page navigation
</script>
```

## Custom Swap Function

Override how the DOM is swapped:

```typescript
import { swapFunctions } from "astro:transitions/client";

document.addEventListener("astro:before-swap", (event) => {
  event.swap = () => {
    swapFunctions.deselectScripts(event.newDocument);
    swapFunctions.swapRootAttributes(event.newDocument);
    swapFunctions.swapHeadElements(event.newDocument);
    const restoreFocus = swapFunctions.saveFocus();
    swapFunctions.swapBodyElement(event.newDocument.body, document.body);
    restoreFocus();
  };
});
```

## Fallback Control

For browsers without View Transition API support:

```astro
<ClientRouter fallback="animate" />  <!-- default: simulate transitions -->
<ClientRouter fallback="swap" />     <!-- instant swap, no animation -->
<ClientRouter fallback="none" />     <!-- full page navigation -->
```

## Accessibility

- **Route announcer**: Automatically announces new page title to screen readers
- **`prefers-reduced-motion`**: All animations disabled when user prefers reduced motion
- Always include a `<title>` on every page

## Security Note

When using `navigate()` with user input, validate the URL against an allowlist to prevent XSS:

```typescript
const allowedPaths = ['/home', '/about', '/contact'];
if (redirect && allowedPaths.includes(redirect)) {
  navigate(redirect);
}
```

Content was rephrased for compliance with licensing restrictions.
