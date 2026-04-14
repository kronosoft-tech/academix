/**
 * Animation Functions using anime.js v4
 * 
 * API v4: animate(targets, { from: x, to: y })
 */

import { animate, set, stagger as animeStagger } from "animejs";

export type AnimeInstance = ReturnType<typeof animate>;

export interface AnimationOptions {
  delay?: number;
  duration?: number;
  easing?: string;
  ease?: string;
  complete?: () => void;
  stagger?: number;
}

function resolveTargets(selector: string | Element | Element[] | null): Element[] {
  if (!selector) return [];
  if (selector instanceof Element) return [selector];
  if (Array.isArray(selector)) return selector.filter((el): el is Element => el instanceof Element);
  if (typeof selector === "string") {
    return Array.from(document.querySelectorAll(selector));
  }
  return [];
}

export function fadeIn(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  const inst = animate(targets, {
    opacity: { from: 0, to: 1 },
    duration: options?.duration ?? 300,
    ease: options?.easing ?? "outQuad",
    delay: options?.delay ?? 0,
  });
  
  if (inst && options?.complete) inst.then(() => options.complete!());
  return inst;
}

export function fadeOut(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  const inst = animate(targets, {
    opacity: { from: 1, to: 0 },
    duration: options?.duration ?? 200,
    ease: options?.easing ?? "inQuad",
    delay: options?.delay ?? 0,
  });
  
  if (inst && options?.complete) inst.then(() => options.complete!());
  return inst;
}

export function slideIn(selector: string | Element | Element[] | null, direction: "left" | "right" | "up" | "down" = "up", options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  const transforms = {
    left: { from: -100, to: 0 },
    right: { from: 100, to: 0 },
    up: { from: 30, to: 0 },
    down: { from: -30, to: 0 },
  }[direction];
  
  const inst = animate(targets, {
    opacity: { from: 0, to: 1 },
    translateX: direction === "left" || direction === "right" ? transforms : { from: 0, to: 0 },
    translateY: direction === "up" || direction === "down" ? transforms : { from: 0, to: 0 },
    duration: options?.duration ?? 300,
    ease: options?.easing ?? "outQuad",
    delay: options?.delay ?? 0,
  });
  
  if (inst && options?.complete) inst.then(() => options.complete!());
  return inst;
}

export function slideOut(selector: string | Element | Element[] | null, direction: "left" | "right" | "up" | "down" = "up", options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  const transforms = {
    left: { from: 0, to: -100 },
    right: { from: 0, to: 100 },
    up: { from: 0, to: -30 },
    down: { from: 0, to: 30 },
  }[direction];
  
  const inst = animate(targets, {
    opacity: { from: 1, to: 0 },
    translateX: direction === "left" || direction === "right" ? transforms : { from: 0, to: 0 },
    translateY: direction === "up" || direction === "down" ? transforms : { from: 0, to: 0 },
    duration: options?.duration ?? 200,
    ease: options?.easing ?? "inQuad",
    delay: options?.delay ?? 0,
  });
  
  if (inst && options?.complete) inst.then(() => options.complete!());
  return inst;
}

export function scaleIn(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  set(targets, { scale: 0, opacity: 0 });
  
  const inst = animate(targets, {
    scale: { from: 0, to: 1 },
    opacity: { from: 0, to: 1 },
    duration: options?.duration ?? 250,
    ease: "outBack",
    delay: options?.delay ?? 0,
  });
  
  if (inst && options?.complete) inst.then(() => options.complete!());
  return inst;
}

export function scaleOut(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  const inst = animate(targets, {
    scale: { from: 1, to: 0 },
    opacity: { from: 1, to: 0 },
    duration: options?.duration ?? 200,
    ease: "inQuad",
    delay: options?.delay ?? 0,
  });
  
  if (inst && options?.complete) inst.then(() => options.complete!());
  return inst;
}

export function stagger(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  const inst = animate(targets, {
    opacity: { from: 0, to: 1 },
    duration: options?.duration ?? 300,
    ease: "outQuad",
    delay: animeStagger(options?.delay ?? 50),
  });
  
  if (inst && options?.complete) inst.then(() => options.complete!());
  return inst;
}

export function countUp(element: Element | null, from: number, to: number, options?: AnimationOptions): AnimeInstance | undefined {
  if (!element) return undefined;
  
  const obj = { value: from };
  const inst = animate(obj, {
    value: to,
    duration: options?.duration ?? 1000,
    ease: "outQuad",
  });
  
  inst.then(() => { element.textContent = String(obj.value); });
  return inst;
}

export function bounce(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  return animate(targets, {
    translateY: [0, -10, 0, -5, 0],
    duration: options?.duration ?? 500,
    ease: "inOutSine",
  });
}

export function pulse(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  return animate(targets, {
    scale: [1, 1.05, 1],
    duration: options?.duration ?? 600,
    ease: "inOutSine",
    loop: true,
    direction: "alternate",
  });
}

export function shake(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  return animate(targets, {
    translateX: [0, -5, 5, -5, 5, 0],
    duration: options?.duration ?? 400,
    ease: "inOutSine",
  });
}

export function animateTableRows(tableBody: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(tableBody);
  if (targets.length === 0) return undefined;
  
  return animate(targets, {
    opacity: { from: 0, to: 1 },
    translateX: [-20, 0],
    duration: options?.duration ?? 300,
    ease: "outQuad",
    delay: animeStagger(options?.delay ?? 50),
  });
}

export const fadeInCards = fadeIn;
export const slideInFromLeft = (s: string | Element | Element[] | null, o?: AnimationOptions) => slideIn(s, "left", o);
export const slideInFromRight = (s: string | Element | Element[] | null, o?: AnimationOptions) => slideIn(s, "right", o);

export const Animations = {
  fadeIn, fadeOut, slideIn, slideOut, scaleIn, scaleOut, stagger, countUp, bounce, pulse, shake, animateTableRows,
};

export default Animations;