# Tasks: Animate Interface

## Phase 1: Infrastructure & Configuration

- [x] 1.1 Fix anime.js v4 ESM import in `package.json` — update import path to use ESM-compatible module resolution
- [x] 1.2 Create `src/lib/animations/config.ts` — export global animation config with duration, easing defaults
- [x] 1.3 Add `prefers-reduced-motion` detection utility in `src/lib/animations/utils.ts`

## Phase 2: Core Animation Functions

- [x] 2.1 Implement `fadeIn()` in `src/lib/animations/functions.ts`
- [x] 2.2 Implement `fadeOut()` in `src/lib/animations/functions.ts`
- [x] 2.3 Implement `slideIn()` (direction: left/right/up/down) in `src/lib/animations/functions.ts`
- [x] 2.4 Implement `slideOut()` in `src/lib/animations/functions.ts`
- [x] 2.5 Implement `scaleIn()` in `src/lib/animations/functions.ts`
- [x] 2.6 Implement `scaleOut()` in `src/lib/animations/functions.ts`
- [x] 2.7 Implement `stagger()` helper for staggered animations in `src/lib/animations/functions.ts`
- [x] 2.8 Implement `countUp()` for number counting animation in `src/lib/animations/functions.ts`

## Phase 3: React Hooks

- [x] 3.1 Create `useEntranceAnimation()` hook using `useLayoutEffect` in `src/lib/animations/hooks/useEntranceAnimation.ts`
- [x] 3.2 Create `useExitAnimation()` hook in `src/lib/animations/hooks/useExitAnimation.ts`
- [x] 3.3 Create `useModalAnimation()` hook for modal entrance/exit in `src/lib/animations/hooks/useModalAnimation.ts`
- [x] 3.4 Create `useStaggeredEntry()` hook for list/staggered animations in `src/lib/animations/hooks/useStaggeredEntry.ts`
- [x] 3.5 Integrate reduced-motion support in all hooks — check `window.matchMedia('(prefers-reduced-motion: reduce)')`

## Phase 4: Component Integration

- [x] 4.1 Refactor `src/shared/ui/components/Modal.tsx` — add entry animation via `useModalAnimation()`
- [x] 4.2 Refactor `src/shared/ui/components/Modal.tsx` — add exit animation on close
- [x] 4.3 Add page transitions in `src/app/layouts/MainLayout.tsx` — animate content on route change
- [x] 4.4 Add staggered entry to `src/pages/Dashboard.tsx` — stagger cards/elements on mount
- [x] 4.5 Add staggered entry to `src/pages/Courses.tsx` — stagger course cards
- [x] 4.6 Add staggered entry to `src/pages/Students.tsx` — stagger student list items

## Phase 5: Testing & Polish

- [ ] 5.1 Verify animations work with `prefers-reduced-motion: reduce` enabled
- [ ] 5.2 Test modal entrance/exit on fast open/close cycles
- [ ] 5.3 Test page transitions don't block navigation
- [ ] 5.4 Test staggered entries render correctly for 10+ items
- [ ] 5.5 Remove any debug console.logs from animation callbacks