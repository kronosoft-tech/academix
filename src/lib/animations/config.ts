/**
 * Global Animation Configuration
 * 
 * Centralized config for all animations in Academix.
 * Supports prefers-reduced-motion and global overrides.
 */

import type { AnimationOptions } from "./functions";

/** Default animation configuration */
export const animationConfig = {
  /** Default duration in milliseconds */
  duration: 300,
  /** Default easing function */
  easing: "easeOutQuad" as const,
  /** Default delay between stagger items */
  staggerDelay: 50,
  /** Default duration for fast animations */
  fastDuration: 150,
  /** Default duration for slow animations */
  slowDuration: 500,
  /** Modal entrance duration */
  modalDuration: 250,
  /** Page transition duration */
  pageTransitionDuration: 200,
};

/** Global animation settings - can be overridden at runtime */
let globalConfig = { ...animationConfig };

/** Check if user prefers reduced motion */
export function prefersReducedMotion(): boolean {
  if (typeof window === "undefined") return false;
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

/** Set global animation config */
export function setAnimationConfig(config: Partial<typeof animationConfig>) {
  globalConfig = { ...globalConfig, ...config };
}

/** Get current animation config */
export function getAnimationConfig() {
  return globalConfig;
}

/** Get effective animation options - respects reduced motion */
export function getEffectiveOptions(options?: AnimationOptions): AnimationOptions {
  if (prefersReducedMotion()) {
    return {
      duration: 0,
      easing: "linear",
      ...options,
    };
  }
  
  return {
    duration: animationConfig.duration,
    easing: animationConfig.easing,
    ...options,
  };
}