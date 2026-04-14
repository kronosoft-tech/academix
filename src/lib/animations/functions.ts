/**
 * Animation Functions using anime.js v4
 * 
 * API v4: animate(targets, options) - targets is FIRST argument
 *         easing -> ease (sin "ease" prefix!)
 */

import { animate, set, stagger as animeStagger } from "animejs";

/** anime.js v4 Animation instance type - simplified */
export type AnimeInstance = ReturnType<typeof animate>;

/** Animation options compatible with anime.js v4 */
export interface AnimationOptions {
  /** Delay in milliseconds */
  delay?: number;
  /** Duration in milliseconds */
  duration?: number;
  /** Easing function name (sin "ease" prefix en v4!) */
  easing?: string;
  /** Callback when animation completes */
  complete?: () => void;
  /** Stagger delay for multiple elements */
  stagger?: number;
}

/** Fade in elements with opacity animation */
export function fadeIn(
  selector: string | Element | Element[] | null,
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  const config = {
    opacity: [0, 1],
    duration: options?.duration ?? 300,
    ease: options?.easing ?? "outQuad",
    delay: options?.delay ?? 0,
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Fade out elements */
export function fadeOut(
  selector: string | Element | Element[] | null,
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  const config = {
    opacity: [1, 0],
    duration: options?.duration ?? 200,
    ease: options?.easing ?? "inQuad",
    delay: options?.delay ?? 0,
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Slide in from a direction */
export function slideIn(
  selector: string | Element | Element[] | null,
  direction: "left" | "right" | "up" | "down" = "up",
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;

  const transforms: Record<string, [number, number]> = {
    left: [-100, 0],
    right: [100, 0],
    up: [50, 0],
    down: [-50, 0],
  };

  const [start, end] = transforms[direction];

  const config = {
    opacity: [0, 1],
    translateX: direction === "left" || direction === "right" ? [start, end] : 0,
    translateY: direction === "up" || direction === "down" ? [start, end] : 0,
    duration: options?.duration ?? 300,
    ease: options?.easing ?? "outQuad",
    delay: options?.delay ?? 0,
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Slide out to a direction */
export function slideOut(
  selector: string | Element | Element[] | null,
  direction: "left" | "right" | "up" | "down" = "up",
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;

  const transforms: Record<string, number> = {
    left: -100,
    right: 100,
    up: -50,
    down: 50,
  };

  const config = {
    opacity: [1, 0],
    translateX: direction === "left" || direction === "right" ? transforms[direction] : 0,
    translateY: direction === "up" || direction === "down" ? transforms[direction] : 0,
    duration: options?.duration ?? 200,
    ease: options?.easing ?? "inQuad",
    delay: options?.delay ?? 0,
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Scale in from 0 to 1 */
export function scaleIn(
  selector: string | Element | Element[] | null,
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;

  // Set initial scale before animating
  set(targets, { scale: 0, opacity: 0 });

  const config = {
    scale: [0, 1],
    opacity: [0, 1],
    duration: options?.duration ?? 250,
    ease: "outBack",
    delay: options?.delay ?? 0,
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Scale out from 1 to 0 */
export function scaleOut(
  selector: string | Element | Element[] | null,
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;

  const config = {
    scale: [1, 0],
    opacity: [1, 0],
    duration: options?.duration ?? 200,
    ease: "inQuad",
    delay: options?.delay ?? 0,
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Staggered animation helper */
export function stagger(
  selector: string | Element | Element[] | null,
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;

  const config = {
    opacity: [0, 1],
    duration: options?.duration ?? 300,
    ease: "outQuad",
    delay: animeStagger(options?.delay ?? 50),
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Count up animation for numbers */
export function countUp(
  element: Element | null,
  from: number,
  to: number,
  options?: AnimationOptions
): AnimeInstance | undefined {
  if (!element) return undefined;

  const obj = { value: from };

  const instance = animate(obj, {
    value: to,
    duration: options?.duration ?? 1000,
    ease: "outQuad",
  });
  
  instance.then(() => {
    element.textContent = String(obj.value);
  });

  return instance as AnimeInstance;
}

/** Bounce effect */
export function bounce(
  selector: string | Element | Element[] | null,
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;

  const config = {
    translateY: [0, -10, 0, -5, 0],
    duration: options?.duration ?? 500,
    ease: "inOutSine",
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Pulse/heartbeat effect */
export function pulse(
  selector: string | Element | Element[] | null,
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;

  const config = {
    scale: [1, 1.05, 1],
    duration: options?.duration ?? 600,
    ease: "inOutSine",
    loop: true,
    direction: "alternate",
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Progress bar fill animation */
export function animateProgressBar(
  selector: string | Element | Element[] | null,
  percentage: number,
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;

  const config = {
    width: [`0%`, `${percentage}%`],
    duration: options?.duration ?? 800,
    ease: "outQuad",
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Shake error effect */
export function shake(
  selector: string | Element | Element[] | null,
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;

  const config = {
    translateX: [0, -5, 5, -5, 5, 0],
    duration: options?.duration ?? 400,
    ease: "inOutSine",
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Layout shift warning */
export function layoutShift(
  selector: string | Element | Element[] | null,
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;

  const config = {
    translateY: [0, -5, 5, -3, 3, 0],
    duration: options?.duration ?? 600,
    ease: "inOutSine",
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

/** Animate table rows with stagger */
export function animateTableRows(
  tableBody: string | Element | Element[] | null,
  options?: AnimationOptions
): AnimeInstance | undefined {
  const targets = resolveTargets(tableBody);
  if (targets.length === 0) return undefined;

  const config = {
    opacity: [0, 1],
    translateX: [-20, 0],
    duration: options?.duration ?? 300,
    ease: "outQuad",
    delay: animeStagger(options?.delay ?? 50),
  };
  
  const instance = animate(targets, config);
  if (options?.complete) {
    instance.then(() => options.complete?.());
  }
  return instance as AnimeInstance;
}

// Aliases for backward compatibility
export const fadeInCards = fadeIn;
export const slideInFromLeft = (s: string | Element | Element[] | null, o?: AnimationOptions) => slideIn(s, "left", o);
export const slideInFromRight = (s: string | Element | Element[] | null, o?: AnimationOptions) => slideIn(s, "right", o);

// Helper to resolve targets
function resolveTargets(selector: string | Element | Element[] | null): Element[] {
  if (!selector) return [];
  if (selector instanceof Element) return [selector];
  if (Array.isArray(selector)) return selector.filter((el): el is Element => el instanceof Element);
  if (typeof selector === "string") {
    const elements = document.querySelectorAll(selector);
    return Array.from(elements);
  }
  return [];
}

// Re-export as default object
export const Animations = {
  fadeIn,
  fadeOut,
  slideIn,
  slideOut,
  scaleIn,
  scaleOut,
  stagger,
  countUp,
  bounce,
  pulse,
  animateProgressBar,
  shake,
  layoutShift,
  animateTableRows,
};

export default Animations;