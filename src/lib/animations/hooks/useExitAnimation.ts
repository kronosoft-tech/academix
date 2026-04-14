/**
 * useExitAnimation Hook
 * 
 * Animates element exit with cleanup callback.
 * Supports prefers-reduced-motion.
 */

import { useEffect, useRef } from "react";
import type { RefObject } from "react";
import { fadeOut, scaleOut, slideOut } from "../functions";
import { prefersReducedMotion } from "../utils";
import { getAnimationConfig } from "../config";

export type ExitType = "fade" | "scale" | "slide-up" | "slide-down" | "slide-left" | "slide-right";

export interface UseExitAnimationOptions {
  /** Type of exit animation */
  type?: ExitType;
  /** Custom duration (overrides global config) */
  duration?: number;
  /** Custom delay */
  delay?: number;
  /** Custom easing */
  easing?: string;
  /** Callback when animation completes - use for DOM removal */
  onComplete?: () => void;
  /** Whether animation is enabled */
  enabled?: boolean;
  /** Trigger the exit animation */
  trigger?: boolean;
}

/**
 * Animate element exit. Calls onComplete after animation finishes.
 * Respects prefers-reduced-motion.
 */
export function useExitAnimation(
  ref: RefObject<HTMLElement | null>,
  options: UseExitAnimationOptions = {}
) {
  const {
    type = "fade",
    duration,
    delay = 0,
    easing,
    onComplete,
    enabled = true,
    trigger = false,
  } = options;

  const hasAnimated = useRef(false);

  useEffect(() => {
    // Only animate when trigger becomes true
    if (!trigger || !enabled || !ref.current || hasAnimated.current) return;
    if (prefersReducedMotion()) {
      // Instant removal for reduced motion
      onComplete?.();
      return;
    }

    const config = getAnimationConfig();
    // Exit animations are typically faster
    const effectiveDuration = duration ?? config.fastDuration;
    const effectiveEasing = easing ?? "easeInQuad";

    hasAnimated.current = true;

    let animation: ReturnType<typeof fadeOut | typeof scaleOut | typeof slideOut> | undefined;

    const handleComplete = () => {
      onComplete?.();
    };

    switch (type) {
      case "scale":
        animation = scaleOut(ref.current, {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: handleComplete,
        });
        break;
      case "slide-up":
        animation = slideOut(ref.current, "up", {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: handleComplete,
        });
        break;
      case "slide-down":
        animation = slideOut(ref.current, "down", {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: handleComplete,
        });
        break;
      case "slide-left":
        animation = slideOut(ref.current, "left", {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: handleComplete,
        });
        break;
      case "slide-right":
        animation = slideOut(ref.current, "right", {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: handleComplete,
        });
        break;
      case "fade":
      default:
        animation = fadeOut(ref.current, {
          duration: effectiveDuration,
          delay,
          easing: effectiveEasing,
          complete: handleComplete,
        });
        break;
    }

    return () => {
      if (animation) {
        animation.pause();
      }
    };
  }, [ref, trigger, type, duration, delay, easing, onComplete, enabled]);
}