/**
 * useStaggeredEntry Hook
 * 
 * Animates multiple elements with staggered delays.
 * Supports prefers-reduced-motion and dynamic item lists.
 */

import { useLayoutEffect, useRef, useEffect } from "react";
import type { RefObject } from "react";
import { stagger as staggerAnimate, slideIn, scaleIn } from "../functions";
import { prefersReducedMotion } from "../utils";
import { getAnimationConfig } from "../config";

export type StaggerType = "fade" | "slide-up" | "slide-down" | "slide-left" | "slide-right" | "scale";

export interface UseStaggeredEntryOptions {
  /** Type of stagger animation */
  type?: StaggerType;
  /** Custom duration per item (overrides global config) */
  duration?: number;
  /** Delay between each item */
  delay?: number;
  /** Custom easing */
  easing?: string;
  /** Callback when all animations complete */
  onComplete?: () => void;
  /** Whether animation is enabled */
  enabled?: boolean;
  /** Number of items - used to calculate stagger positions */
  itemCount?: number;
}

/**
 * Staggered entry animation for multiple elements.
 * Respects prefers-reduced-motion.
 * 
 * IMPORTANT: This hook expects items to have data-index attributes
 * or be direct children that can be selected with the selector.
 */
export function useStaggeredEntry(
  ref: RefObject<HTMLElement | null>,
  options: UseStaggeredEntryOptions = {}
) {
  const {
    type = "fade",
    duration,
    delay,
    easing,
    onComplete,
    enabled = true,
    itemCount,
  } = options;

  const hasAnimated = useRef(false);
  const animationRef = useRef<ReturnType<typeof staggerAnimate> | null>(null);

  useLayoutEffect(() => {
    if (!enabled || !ref.current || hasAnimated.current) return;
    if (prefersReducedMotion()) {
      // Instant appearance for reduced motion
      const children = ref.current.children;
      Array.from(children).forEach((child) => {
        (child as HTMLElement).style.opacity = "1";
        (child as HTMLElement).style.transform = "none";
      });
      onComplete?.();
      return;
    }

    const config = getAnimationConfig();
    const effectiveDuration = duration ?? config.duration;
    const effectiveDelay = delay ?? config.staggerDelay;
    const effectiveEasing = easing ?? config.easing;

    hasAnimated.current = true;

    // Get direct children as targets
    const children = ref.current.children;
    if (children.length === 0) {
      onComplete?.();
      return;
    }

    const targets = Array.from(children);

    switch (type) {
      case "slide-up":
        animationRef.current = slideIn(targets, "up", {
          duration: effectiveDuration,
          delay: effectiveDelay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
      case "slide-down":
        animationRef.current = slideIn(targets, "down", {
          duration: effectiveDuration,
          delay: effectiveDelay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
      case "slide-left":
        animationRef.current = slideIn(targets, "left", {
          duration: effectiveDuration,
          delay: effectiveDelay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
      case "slide-right":
        animationRef.current = slideIn(targets, "right", {
          duration: effectiveDuration,
          delay: effectiveDelay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
      case "scale":
        // Set initial scale for scale animation
        targets.forEach((el) => {
          (el as HTMLElement).style.opacity = "0";
          (el as HTMLElement).style.transform = "scale(0)";
        });
        animationRef.current = scaleIn(targets, {
          duration: effectiveDuration,
          delay: effectiveDelay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
      case "fade":
      default:
        animationRef.current = staggerAnimate(targets, {
          duration: effectiveDuration,
          delay: effectiveDelay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
    }

    return () => {
      if (animationRef.current) {
        animationRef.current.pause();
      }
    };
  }, [ref, type, duration, delay, easing, onComplete, enabled, itemCount]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (animationRef.current) {
        animationRef.current.pause();
      }
      if (ref.current) {
        const children = ref.current.children;
        Array.from(children).forEach((child) => {
          (child as HTMLElement).style.opacity = "";
          (child as HTMLElement).style.transform = "";
        });
      }
    };
  }, [ref]);
}