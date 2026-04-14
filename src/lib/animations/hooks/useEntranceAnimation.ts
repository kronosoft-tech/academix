/**
 * useEntranceAnimation Hook
 * 
 * Animates element entrance using useLayoutEffect to prevent flash.
 * Supports prefers-reduced-motion.
 */

import { useLayoutEffect, useRef, useEffect } from "react";
import type { RefObject } from "react";
import { fadeIn, scaleIn, slideIn } from "../functions";
import { prefersReducedMotion } from "../utils";
import { getAnimationConfig } from "../config";

export type EntranceType = "fade" | "scale" | "slide-up" | "slide-down" | "slide-left" | "slide-right";

export interface UseEntranceAnimationOptions {
  /** Type of entrance animation */
  type?: EntranceType;
  /** Custom duration (overrides global config) */
  duration?: number;
  /** Custom delay */
  delay?: number;
  /** Custom easing */
  easing?: string;
  /** Callback when animation completes */
  onComplete?: () => void;
  /** Whether animation is enabled */
  enabled?: boolean;
}

/**
 * Animate element on mount using useLayoutEffect to prevent flash.
 * Respects prefers-reduced-motion.
 */
export function useEntranceAnimation(
  ref: RefObject<HTMLElement | null>,
  options: UseEntranceAnimationOptions = {}
) {
  const {
    type = "fade",
    duration,
    delay = 0,
    easing,
    onComplete,
    enabled = true,
  } = options;

  const hasAnimated = useRef(false);

  useLayoutEffect(() => {
    if (!enabled || !ref.current || hasAnimated.current) return;
    if (prefersReducedMotion()) {
      // Instant appearance for reduced motion
      if (ref.current) {
        ref.current.style.opacity = "1";
        ref.current.style.transform = "none";
      }
      onComplete?.();
      return;
    }

    const config = getAnimationConfig();
    const effectiveDuration = duration ?? config.duration;
    const effectiveEasing = easing ?? config.easing;

    hasAnimated.current = true;

    let animation: ReturnType<typeof fadeIn | typeof scaleIn | typeof slideIn> | undefined;

    switch (type) {
      case "scale":
        animation = scaleIn(ref.current, {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
      case "slide-up":
        animation = slideIn(ref.current, "up", {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
      case "slide-down":
        animation = slideIn(ref.current, "down", {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
      case "slide-left":
        animation = slideIn(ref.current, "left", {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
      case "slide-right":
        animation = slideIn(ref.current, "right", {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
      case "fade":
      default:
        animation = fadeIn(ref.current, {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: onComplete,
        });
        break;
    }

    return () => {
      // Cleanup - v4 uses different API
      try {
        (animation as unknown as { cancel: () => void })?.cancel?.();
      } catch {
        // Ignore cleanup errors
      }
    };
  }, [ref, type, duration, delay, easing, onComplete, enabled]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (ref.current) {
        // Remove any animation state
        ref.current.style.opacity = "";
        ref.current.style.transform = "";
      }
    };
  }, [ref]);
}