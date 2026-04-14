/**
 * Animation Utilities
 * 
 * Shared utilities for animation detection and helper functions.
 */

/** Check if user prefers reduced motion */
export function prefersReducedMotion(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/** Listen for changes in prefers-reduced-motion */
export function onReducedMotionChange(callback: (reduced: boolean) => void): () => void {
  if (typeof window === "undefined") {
    return () => {};
  }

  const mediaQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
  
  const handler = (e: MediaQueryListEvent) => {
    callback(e.matches);
  };

  mediaQuery.addEventListener("change", handler);
  return () => mediaQuery.removeEventListener("change", handler);
}

/** Convert selector to Element or Element[] */
export function resolveTargets(selector: string | Element | Element[] | null): Element[] {
  if (!selector) return [];
  
  if (selector instanceof Element) {
    return [selector];
  }
  
  if (Array.isArray(selector)) {
    return selector.filter((el): el is Element => el instanceof Element);
  }
  
  // It's a string selector
  if (typeof selector === "string") {
    const elements = document.querySelectorAll(selector);
    return Array.from(elements);
  }
  
  return [];
}

/** Wait for the next animation frame */
export function animationFrame(): Promise<void> {
  return new Promise((resolve) => {
    requestAnimationFrame(() => resolve());
  });
}