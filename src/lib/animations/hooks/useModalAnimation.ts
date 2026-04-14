/**
 * useModalAnimation Hook
 * 
 * Specialized hook for modal entrance/exit animations.
 * Uses useLayoutEffect for entrance to prevent flash.
 * Supports prefers-reduced-motion.
 */

import { useState, useRef, useLayoutEffect, useEffect } from "react";
import type { RefObject } from "react";
import { fadeIn, fadeOut, scaleIn } from "../functions";
import { prefersReducedMotion } from "../config";
import { getAnimationConfig } from "../config";

export interface UseModalAnimationOptions {
  /** Modal is open */
  isOpen: boolean;
  /** Backdrop ref */
  backdropRef?: RefObject<HTMLElement | null>;
  /** Modal content ref */
  modalRef?: RefObject<HTMLElement | null>;
  /** Entrance animation type */
  entranceType?: "scale" | "fade";
  /** Custom entrance duration */
  duration?: number;
  /** Custom entrance delay */
  delay?: number;
  /** Callback when entrance animation completes */
  onEnterComplete?: () => void;
  /** Callback when exit animation completes */
  onExitComplete?: () => void;
}

/**
 * Modal animation hook with entrance and exit animations.
 * 
 * - Entrance: uses useLayoutEffect (synchronous) to prevent flash
 * - Exit: uses useEffect with trigger pattern
 * - Respects prefers-reduced-motion
 */
export function useModalAnimation(options: UseModalAnimationOptions) {
  const {
    isOpen,
    backdropRef,
    modalRef,
    entranceType = "scale",
    duration,
    delay = 0,
    onEnterComplete,
    onExitComplete,
  } = options;

  const [isExiting, setIsExiting] = useState(false);
  const hasEntered = useRef(false);
  const hasExited = useRef(false);

  // Entrance animation - useLayoutEffect to prevent flash
  useLayoutEffect(() => {
    if (!isOpen || hasEntered.current) return;
    if (prefersReducedMotion()) {
      // Instant appearance for reduced motion
      if (backdropRef?.current) {
        backdropRef.current.style.opacity = "1";
      }
      if (modalRef?.current) {
        modalRef.current.style.opacity = "1";
        modalRef.current.style.transform = "none";
      }
      hasEntered.current = true;
      onEnterComplete?.();
      return;
    }

    const config = getAnimationConfig();
    const effectiveDuration = duration ?? config.modalDuration;

// Animate backdrop - just fade in from current state
      if (backdropRef?.current) {
        fadeIn(backdropRef.current, {
          duration: effectiveDuration,
          delay,
          ease: "outQuad",
        });
      }

      // Animate modal content
      if (modalRef?.current) {
        if (entranceType === "scale") {
          scaleIn(modalRef.current, {
            duration: effectiveDuration,
            delay: delay + 50, // Slight delay after backdrop
            easing: "outBack",
            complete: () => {
              hasEntered.current = true;
              onEnterComplete?.();
            },
          });
        } else {
          fadeIn(modalRef.current, {
            duration: effectiveDuration,
            delay: delay + 50,
            ease: "outQuad",
            complete: () => {
              hasEntered.current = true;
              onEnterComplete?.();
            },
          });
        }
      }

    // If no modal ref, still mark as entered
    if (!modalRef?.current) {
      hasEntered.current = true;
      onEnterComplete?.();
    }
  }, [isOpen, backdropRef, modalRef, entranceType, duration, delay, onEnterComplete]);

  // Exit animation
  useEffect(() => {
    // When closing, trigger exit animation
    if (!isOpen && hasEntered.current && !hasExited.current) {
      setIsExiting(true);
    }

    if (!isOpen && hasEntered.current && isExiting) {
      if (prefersReducedMotion()) {
        // Instant removal for reduced motion
        setIsExiting(false);
        hasExited.current = true;
        hasEntered.current = false;
        onExitComplete?.();
        return;
      }

      const config = getAnimationConfig();
      const effectiveDuration = duration ?? config.fastDuration;

      // Animate backdrop
      if (backdropRef?.current) {
        fadeOut(backdropRef.current, {
          duration: effectiveDuration,
          easing: "easeInQuad",
        });
      }

      // Animate modal content
      if (modalRef?.current) {
        fadeOut(modalRef.current, {
          duration: effectiveDuration,
          delay: 0,
          easing: "easeInQuad",
          complete: () => {
            setIsExiting(false);
            hasExited.current = true;
            hasEntered.current = false;
            onExitComplete?.();
          },
        });
      } else {
        // If no modal ref, just wait for backdrop
        setTimeout(() => {
          setIsExiting(false);
          hasExited.current = true;
          hasEntered.current = false;
          onExitComplete?.();
        }, effectiveDuration);
      }
    }

    // Reset when opening again
    if (isOpen) {
      hasExited.current = false;
    }
  }, [isOpen, backdropRef, modalRef, duration, isExiting, onExitComplete]);

  return {
    isExiting,
    shouldRender: isOpen || isExiting,
  };
}