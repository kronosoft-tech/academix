# Design: Animate Interface - anime.js v4 Integration

## Technical Approach

Implementation of anime.js v4 for interface animations in Academix. The strategy involves:
1. Solving ESM import issues with anime.js v4 in Vite/React 19
2. Building a thin wrapper layer that abstracts anime.js for React integration
3. Creating reusable hooks for component animations
4. Integrating animations into Modal and MainLayout components

This approach differs from typical React animation libraries by keeping anime.js as the animation engine while adding React-friendly abstractions.

---

## Architecture Decisions

### Decision: anime.js v4 Import Strategy

**Choice**: Use named import with dynamic import() fallback pattern

```typescript
// animations.ts - Primary import pattern
import anime from 'animejs/lib/anime.esm.js';
```

**Alternatives considered**:
- `import * as anime from 'animejs'` — namespace import
- `import { anime } from 'animejs'` — named export (unlikely to work with v4)
- Dynamic `import('animejs')` — lazy loading

**Rationale**: 
- anime.js v4 distributes as ESM with a default export
- Using `anime from 'animejs/lib/anime.esm.js` provides direct ESM access
- Dynamic import fallback handles edge cases where bundler resolution fails
- This pattern is explicit and debuggable

### Decision: Animation API Structure

**Choice**: Functional utilities + React hooks (not a custom animation component library)

**Alternatives considered**:
- Create `<AnimatedContainer>` wrapper components
- Build a complete animation component library like framer-motion
- Use CSS-only animations with keyframes

**Rationale**:
- The existing codebase already has `animations.ts` with planned functions
- Adding hooks is more ergonomic than wrapper components for this use case
- Keep animations simple and reusable across different contexts
- Avoids over-engineering when only Modal and MainLayout need animations

### Decision: React Integration Pattern

**Choice**: Hooks with refs + useEffect cleanup

```typescript
// Example hook pattern
export function useFadeIn(ref: RefObject<HTMLElement>, options?: AnimationConfig) {
  useEffect(() => {
    if (!ref.current) return;
    anime({ targets: ref.current, opacity: [0, 1], ...options });
    return () => anime.remove(ref.current);
  }, [dependencies]);
}
```

**Alternatives considered**:
- useLayoutEffect for synchronous animations
- CSS classes + anime.js for class-based triggers

**Rationale**:
- useEffect runs after paint, preventing layout thrashing
- Cleanup with `anime.remove()` prevents memory leaks in React 19
- Ref-based targeting is more reliable than query selectors

### Decision: Modal Animation Strategy

**Choice**: Scale + fade for open, fade only for close

**Alternatives considered**:
- Slide + fade from bottom (modals typically center)
- Flip animation for complex transitions

**Rationale**:
- Scale + fade is the industry standard for modals (seen in Material UI, Radix, Headless UI)
- Fade-only close feels faster and less blocking
- Maintains accessibility (users can still see backdrop before it's gone)

### Decision: MainLayout Page Transition

**Choice**: Simple fade on page change

**Alternatives considered**:
- Animated route transitions with React Router
- Slide between pages

**Rationale**:
- The layout uses state-based rendering (no React Router yet)
- Adding a fade transition is low-effort, high-impact
- Future: can add slide when routing is implemented

---

## Data Flow

### Animation Module Structure

```
src/features/accounting/lib/animations.ts
    │
    ├── anime.js v4 wrapper (handles ESM import)
    │
    ├── Functional utilities
    │   ├── fadeInCards(), fadeOut()
    │   ├── slideInFromLeft/Right()
    │   ├── scaleIn/Out()
    │   ├── countUp(), bounce(), pulse()
    │   └── animateTableRows(), shake(), layoutShift()
    │
    └── Re-exports as AccountingAnimations object
         │
         ▼
src/hooks/
    ├── useAnimation.ts      (generic hook)
    ├── useFadeIn.ts      (specific hooks)
    ├── useFadeOut.ts
    ├── useScaleIn.ts
    └── usePageTransition.ts
         │
         ▼
src/shared/ui/components/Modal.tsx    ← Uses useScaleIn + useFadeOut
src/app/layouts/MainLayout.tsx     ← Uses usePageTransition
```

### Component Animation Flow

```
Modal.tsx:
  isOpen: true
     │
     ├─► useScaleIn(ref, { delay: 0 })     → animate scale 0→1, opacity 0→1
     │
     └─► Backdrop uses useFadeIn            → animate opacity 0→0.5

  isOpen: false
     │
     └─► useFadeOut({ complete: onClose })  → animate opacity → 0, then remove from DOM

MainLayout.tsx:
  currentPage state changes
     │
     └─► usePageTransition(ref, currentPage) → animate opacity, slight scale on page change
```

---

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/features/accounting/lib/anime-esm.ts` | Create | anime.js v4 ESM wrapper with fallback import |
| `src/features/accounting/lib/animations.ts` | Modify | Implement all placeholder functions using anime-esm |
| `src/hooks/useAnimation.ts` | Create | Generic animation hook |
| `src/hooks/useFadeIn.ts` | Create | Fade-in hook |
| `src/hooks/useFadeOut.ts` | Create | Fade-out hook |
| `src/hooks/useScaleIn.ts` | Create | Scale-in hook (for Modal) |
| `src/hooks/usePageTransition.ts` | Create | Page transition hook (for MainLayout) |
| `src/shared/ui/components/Modal.tsx` | Modify | Integrate scale/fade animations |
| `src/app/layouts/MainLayout.tsx` | Modify | Add page transition animation |
| `openspec/changes/animate-interface/design.md` | Create | This design document |

---

## Interfaces / Contracts

### Animation Types

```typescript
// src/features/accounting/lib/animations.ts

interface AnimationConfig {
  /** Delay in milliseconds */
  delay?: number;
  /** Duration in milliseconds */
  duration?: number;
  /** Easing function name */
  easing?: "easeInOutQuad" | "easeOutQuad" | "spring" | string;
  /** Callback when animation completes */
  complete?: () => void;
}

/** anime.js v4 wrapper with ESM import handling */
export function getAnime(): Promise<typeof anime>;

/** Fade in elements with opacity animation */
export function fadeInCards(
  selector: string | Element | Element[],
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Fade out elements */
export function fadeOut(
  selector: string | Element | Element[],
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Slide from left (translateX) */
export function slideInFromLeft(
  selector: string | Element | Element[],
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Slide from right */
export function slideInFromRight(
  selector: string | Element | Element[],
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Scale from 0 to 1 */
export function scaleIn(
  selector: string | Element | Element[],
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Scale from 1 to 0 */
export function scaleOut(
  selector: string | Element | Element[],
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Count up animation for numbers */
export function countUp(
  element: Element | null,
  from: number,
  to: number,
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Staggered row animation for tables */
export function animateTableRows(
  tableBody: string | Element | Element[],
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Bounce effect */
export function bounce(
  selector: string | Element | Element[],
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Pulse/heartbeat effect */
export function pulse(
  selector: string | Element | Element[],
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Progress bar fill */
export function animateProgressBar(
  selector: string | Element | Element[],
  percentage: number,
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Shake error effect */
export function shake(
  selector: string | Element | Element[],
  options?: AnimationConfig
): anime.AnimeInstance | undefined;

/** Layout shift warning */
export function layoutShift(
  selector: string | Element | Element[],
  options?: AnimationConfig
): anime.AnimeInstance | undefined;
```

### React Hooks

```typescript
// src/hooks/useAnimation.ts

import { RefObject, useEffect, useRef } from "react";
import { getAnime } from "../features/accounting/lib/animations";

interface UseAnimationOptions extends AnimationConfig {
  /** Trigger animation when condition is true */
  trigger?: boolean;
}

export function useAnimation(
  ref: RefObject<HTMLElement>,
  animation: (targets: any) => object,  // anime.js config
  options?: UseAnimationOptions
): void;
```

```typescript
// src/hooks/useFadeIn.ts

import { RefObject } from "react";
import { AnimationConfig } from "../features/accounting/lib/animations";

interface UseFadeInOptions extends AnimationConfig {
  /** Dependencies that trigger re-animation */
  deps?: unknown[];
}

export function useFadeIn(
  ref: RefObject<HTMLElement>,
  options?: UseFadeInOptions
): void;
```

```typescript
// src/hooks/useScaleIn.ts

import { RefObject } from "react";
import { AnimationConfig } from "../features/accounting/lib/animations";

interface UseScaleInOptions extends AnimationConfig {
  /** Trigger when this becomes true */
  trigger?: boolean;
}

export function useScaleIn(
  ref: RefObject<HTMLElement>,
  options?: UseScaleInOptions
): void;
```

### Modal Integration

```typescript
// src/shared/ui/components/Modal.tsx (modified interface)

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: ReactNode;
  size?: "sm" | "md" | "lg" | "xl";
  /** Disable entrance animation */
  animate?: boolean;
}

// Inside component:
// - backdrop ref → useFadeIn on isOpen
// - modal ref → useScaleIn on isOpen
```

---

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | Animation functions work | Test with jsdom + anime.js, verify targets animate |
| Unit | Hooks call anime | Mock anime, verify correct config passed |
| Integration | Modal animates on open/close | Playwright: click open, verify animation class present |
| Integration | MainLayout page transition | Playwright: click nav, verify fade occurs |

### Test Files to Create

```
tests/
├── unit/
│   ├── animations.test.ts
│   └── hooks/
│       ├── useFadeIn.test.ts
│       └── useScaleIn.test.ts
└── e2e/
    ├── modal-animation.spec.ts
    └── layout-transition.spec.ts
```

---

## Migration / Rollback

No migration required. This is a net-new feature implementation.

### Rollback Plan

1. **Keep placeholders**: If ESM issues persist, leave animations.ts as placeholders with comments explaining the issue
2. **Remove hooks**: If hooks cause memory leaks, remove `src/hooks/use*.ts` files
3. **Revert components**: Revert Modal.tsx and MainLayout.tsx to non-animated versions

Rollback command:
```bash
git checkout HEAD -- src/features/accounting/lib/animations.ts
git checkout HEAD -- src/shared/ui/components/Modal.tsx  
git checkout HEAD -- src/app/layouts/MainLayout.tsx
rm -rf src/hooks/useAnimation.ts src/hooks/useFadeIn.ts src/hooks/useFadeOut.ts src/hooks/useScaleIn.ts src/hooks/usePageTransition.ts
```

---

## Implementation Order

1. **Phase 1: Fix ESM import**
   - Create `anime-esm.ts` wrapper
   - Verify import works in dev mode

2. **Phase 2: Implement animations.ts**
   - Fill in all placeholder functions
   - Test each function manually

3. **Phase 3: Create hooks**
   - Create basic hooks
   - Test in isolation

4. **Phase 4: Integrate Modal**
   - Add refs and hooks
   - Test open/close animations

5. **Phase 5: Integrate MainLayout**
   - Add page transition hook
   - Verify no jank on nav click

---

## Open Questions

- [ ] **Q1**: Should we use `useLayoutEffect` instead of `useEffect` for modal entrance to prevent flash?
  - **A1**: Start with `useEffect`, switch if visible flash occurs during mount

- [ ] **Q2**: Do we need to support reduced-motion preference?
  - **A2**: Add `prefers-reduced-motion` check in hooks, skip animation if true

- [ ] **Q3**: Should animations be configurable globally?
  - **A3**: Consider adding a simple config object later if needed

- [ ] **Q4**: Are there other components that would benefit from animations?
  - **A4**: Survey after initial implementation, prioritize based on user feedback