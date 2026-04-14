# Exploration: animate-interface

## Current State

The Academix application has **zero active animation implementation**:

### Animation Library Status
- **animejs** is installed in `package.json` (v4.3.6) but is **NOT being used**
- The `src/features/accounting/lib/animations.ts` file contains **only placeholder functions** (comment: "anime.js v4 ESM issues")
- No GSAP, framer-motion, or motion library usage found anywhere in the codebase

### Page/Route Architecture
- Uses **react-router-dom v7** with hash routing (`createHashRouter`)
- BUT the MainLayout uses a **custom state-based navigation** (`useState<Page>`) NOT React Router's navigation
- Pages render via `renderPage()` switch statement — this is significant for page transitions
- All pages are lazy-loaded via the router

### Modal Implementation
- Simple `Modal` component in `src/shared/ui/components/Modal.tsx`
- **No animations** — just conditional rendering with basic CSS transitions on backdrop
- Uses inline conditional rendering (`if (!isOpen) return null`)

### CSS Animation Utilities
- Tailwind CSS v4 with custom theme colors
- Basic hover transitions in buttons (CSS `transition-colors`)
- Modal backdrop has `transition-opacity` only
- No keyframe animations defined

## Affected Areas

| File/Component | Why Affected |
|----------------|--------------|
| `src/features/accounting/lib/animations.ts` | Placeholder functions need real implementation |
| `src/app/layouts/MainLayout.tsx` | Page transitions via state-based navigation |
| `src/shared/ui/components/Modal.tsx` | Needs enter/exit animations |
| All page components | Need entry animations (fade-in, stagger) |
| `src/features/*/routes/*.tsx` | Interactive feedback (button press, form validation) |
| `package.json` | anime.js already installed but broken (v4 ESM) |

## Approaches

### 1. Fix anime.js v4 Integration
- **Pros**: Already in dependencies, extensive API
- **Cons**: v4 has ESM issues, different API than v3, may require workaround
- **Effort**: Medium

### 2. Use CSS-only Animations
- **Pros**: No dependencies, performant, works with existing transitions
- **Cons**: Limited capabilities, can't do complex sequences
- **Effort**: Low

### 3. Add GSAP (替代方案)
- **Pros**: Powerful, well-maintained, works with React refs
- **Cons**: Additional dependency, learning curve
- **Effort**: Medium-High

### 4. Hybrid: CSS + Minimal JS
- **Pros**: Best performance, progressive enhancement
- **Cons**: May not meet "dynamic and beautiful" requirement alone
- **Effort**: Low-Medium

## Recommendation

**Use anime.js v3 (downgrade from v4)** to fix ESM issues while maintaining the existing placeholder API structure. The animations.ts file already defines the exact functions needed:
- `fadeInCards`, `fadeOut`
- `slideInFromLeft`, `slideInFromRight`  
- `scaleIn`, `scaleOut`
- `countUp`, `animateTableRows`
- `bounce`, `pulse`, `shake`, `layoutShift`, `animateProgressBar`

This approach:
1. Uses existing dependency (no new installs)
2. Preserves the planned API from the placeholder file
3. Enables all requested animation types
4. Works with React refs and selectors

**Additional needs**:
- Wrap Modal with animation component (AnimatePresence-like pattern)
- Add page transition wrapper in MainLayout
- Create animation hooks for reusable patterns

## Risks

- **anime.js v4 ESM issues**: Need to downgrade to v3.x or use dynamic import workaround
- **State-based navigation**: Page transitions must hook into `setCurrentPage()` — no router navigation events to latch onto
- **No animation hooks exist**: Need to create `useAnimation` patterns for components
- **Modal unmount timing**: Current inline conditional needs refactor for exit animations

## Ready for Proposal

**Yes** — The exploration reveals:
- anime.js is already installed (v4 problem is solvable)
- Clear API surface defined in placeholder file
- MainLayout state-based routing needs custom transition logic
- Modal needs refactor for enter/exit animations
- All 5 animation categories from user request are covered by existing placeholder functions