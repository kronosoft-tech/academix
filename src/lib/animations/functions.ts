/**
 * Animation Functions using anime.js v4
 * 
 * Based on official docs: https://animejs.com/documentation
 * 
 * Syntax: animate(targets, { property: [from, to], duration, ease })
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

// Base animation helper
function runAnim(
  targets: Element[],
  props: Record<string, unknown>,
  options?: AnimationOptions
): AnimeInstance | undefined {
  if (targets.length === 0) return undefined;
  
  const config = {
    ...props,
    duration: options?.duration ?? 300,
    ease: options?.easing ?? options?.ease ?? "outQuad",
    delay: options?.delay ?? 0,
  };
  
  const inst = animate(targets, config);
  
  if (inst && options?.complete) {
    inst.then(() => options.complete!());
  }
  
  return inst;
}

export function fadeIn(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  return runAnim(targets, { opacity: [0, 1] }, options);
}

export function fadeOut(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  return runAnim(targets, { opacity: [1, 0] }, { ...options, duration: options?.duration ?? 200, ease: options?.easing ?? "inQuad" });
}

export function slideIn(selector: string | Element | Element[] | null, direction: "left" | "right" | "up" | "down" = "up", options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  const transforms: Record<string, number> = {
    left: -100,
    right: 100,
    up: 30,
    down: -30,
  };
  
  const offset = transforms[direction];
  const props = direction === "left" || direction === "right"
    ? { opacity: [0, 1], translateX: [offset, 0] }
    : { opacity: [0, 1], translateY: [offset, 0] };
  
  return runAnim(targets, props, options);
}

export function slideOut(selector: string | Element | Element[] | null, direction: "left" | "right" | "up" | "down" = "up", options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  const transforms: Record<string, number> = {
    left: -100,
    right: 100,
    up: -30,
    down: 30,
  };
  
  const offset = transforms[direction];
  const props = direction === "left" || direction === "right"
    ? { opacity: [1, 0], translateX: [0, offset] }
    : { opacity: [1, 0], translateY: [0, offset] };
  
  return runAnim(targets, props, { ...options, duration: options?.duration ?? 200, ease: "inQuad" });
}

export function scaleIn(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  if (targets.length === 0) return undefined;
  
  set(targets, { scale: 0, opacity: 0 });
  return runAnim(targets, { scale: [0, 1], opacity: [0, 1] }, { ...options, ease: "outBack" });
}

export function scaleOut(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  return runAnim(targets, { scale: [1, 0], opacity: [1, 0] }, { ...options, duration: options?.duration ?? 200 });
}

export function stagger(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  return runAnim(targets, { 
    opacity: [0, 1],
    delay: animeStagger(options?.delay ?? 50)
  }, options);
}

export function countUp(element: Element | null, from: number, to: number, options?: AnimationOptions): AnimeInstance | undefined {
  if (!element) return undefined;
  
  const obj = { value: from };
  const inst = animate(obj, { value: to, duration: options?.duration ?? 1000, ease: "outQuad" });
  inst.then(() => { element.textContent = String(obj.value); });
  return inst;
}

export function bounce(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  return runAnim(targets, { translateY: [0, -10, 0, -5, 0] }, { ...options, ease: "inOutSine" });
}

export function pulse(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  return runAnim(targets, { scale: [1, 1.05, 1], loop: true, direction: "alternate" }, { ...options, ease: "inOutSine" });
}

export function shake(selector: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(selector);
  return runAnim(targets, { translateX: [0, -5, 5, -5, 5, 0] }, { ...options, ease: "inOutSine" });
}

export function animateTableRows(tableBody: string | Element | Element[] | null, options?: AnimationOptions): AnimeInstance | undefined {
  const targets = resolveTargets(tableBody);
  return runAnim(targets, { 
    opacity: [0, 1], 
    translateX: [-20, 0],
    delay: animeStagger(options?.delay ?? 50)
  }, { ...options, ease: "outQuad" });
}

export const fadeInCards = fadeIn;
export const slideInFromLeft = (s: string | Element | Element[] | null, o?: AnimationOptions) => slideIn(s, "left", o);
export const slideInFromRight = (s: string | Element | Element[] | null, o?: AnimationOptions) => slideIn(s, "right", o);

export const Animations = {
  fadeIn, fadeOut, slideIn, slideOut, scaleIn, scaleOut, stagger, countUp, bounce, pulse, shake, animateTableRows,
};

export default Animations;