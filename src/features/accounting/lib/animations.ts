// Animation Utilities - Phase 12
// Anime.js animations for accounting module

// eslint-disable-next-line @typescript-eslint/no-require-imports
const anime = require("animejs");

// Animation configurations
const DEFAULT_DURATION = 400;
const STAGGER_DELAY = 50;

// Fade in animation for cards
export function fadeInCards(
  selector: string | Element | NodeListOf<Element>,
  options?: {
    delay?: number;
    duration?: number;
    easing?: string;
  }
) {
  const { delay = STAGGER_DELAY, duration = DEFAULT_DURATION, easing = "easeOutQuad" } = options || {};

  return anime({
    targets: selector,
    opacity: [0, 1],
    translateY: [20, 0],
    duration,
    delay: anime.stagger(delay),
    easing,
  });
}

// Fade out animation
export function fadeOut(
  selector: string | Element | NodeListOf<Element>,
  options?: {
    duration?: number;
    easing?: string;
    complete?: () => void;
  }
) {
  const { duration = DEFAULT_DURATION, easing = "easeInQuad", complete } = options || {};

  return anime({
    targets: selector,
    opacity: 0,
    translateY: -10,
    duration,
    easing,
    complete,
  });
}

// Slide in from left
export function slideInFromLeft(
  selector: string | Element | NodeListOf<Element>,
  options?: {
    delay?: number;
    duration?: number;
    easing?: string;
  }
) {
  const { delay = 0, duration = 500, easing = "spring(1, 80, 10, 0)" } = options || {};

  return anime({
    targets: selector,
    translateX: [-100, 0],
    opacity: [0, 1],
    duration,
    delay,
    easing,
  });
}

// Slide in from right
export function slideInFromRight(
  selector: string | Element | NodeListOf<Element>,
  options?: {
    delay?: number;
    duration?: number;
    easing?: string;
  }
) {
  const { delay = 0, duration = 500, easing = "spring(1, 80, 10, 0)" } = options || {};

  return anime({
    targets: selector,
    translateX: [100, 0],
    opacity: [0, 1],
    duration,
    delay,
    easing,
  });
}

// Scale up animation (for modals)
export function scaleIn(
  selector: string | Element | NodeListOf<Element>,
  options?: {
    duration?: number;
    easing?: string;
  }
) {
  const { duration = 300, easing = "easeOutBack" } = options || {};

  return anime({
    targets: selector,
    scale: [0.8, 1],
    opacity: [0, 1],
    duration,
    easing,
  });
}

// Scale down animation (for closing modals)
export function scaleOut(
  selector: string | Element | NodeListOf<Element>,
  options?: {
    duration?: number;
    easing?: string;
    complete?: () => void;
  }
) {
  const { duration = 200, easing = "easeInBack", complete } = options || {};

  return anime({
    targets: selector,
    scale: [1, 0.9],
    opacity: [1, 0],
    duration,
    easing,
    complete,
  });
}

// Number count up animation (for stats)
export function countUp(
  element: Element | null,
  from: number,
  to: number,
  options?: {
    duration?: number;
    easing?: string;
    complete?: () => void;
  }
) {
  if (!element) return;

  const { duration = 1000, easing = "easeOutQuad", complete } = options || {};

  const obj = { value: from };

  return anime({
    targets: obj,
    value: to,
    round: 100, // 2 decimal places
    duration,
    easing,
    update: () => {
      element.textContent = `S/ ${obj.value.toLocaleString("es-PE", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
    },
    complete,
  });
}

// Staggered row animation for tables
export function animateTableRows(
  tableBody: string | Element | NodeListOf<Element>,
  options?: {
    stagger?: number;
    duration?: number;
  }
) {
  const { stagger = 30, duration = 400 } = options || {};

  return anime({
    targets: tableBody,
    opacity: [0, 1],
    translateY: [10, 0],
    duration,
    delay: anime.stagger(stagger),
    easing: "easeOutQuad",
  });
}

// Bounce animation for buttons
export function bounce(
  selector: string | Element | NodeListOf<Element>,
  options?: {
    duration?: number;
    scale?: number;
  }
) {
  const { duration = 300, scale = 1.05 } = options || {};

  return anime({
    targets: selector,
    scale: [1, scale, 1],
    duration,
    easing: "easeInOutSine",
  });
}

// Pulse animation for loading states
export function pulse(
  selector: string | Element | NodeListOf<Element>,
  options?: {
    direction?: "in" | "out" | "both";
    duration?: number;
  }
) {
  const { direction = "both", duration = 1000 } = options || {};

  return anime({
    targets: selector,
    opacity: direction === "in" || direction === "both" ? [0.5, 1] : [1, 0.5],
    scale: direction === "in" || direction === "both" ? [0.98, 1] : [1, 0.98],
    duration,
    direction: direction === "both" ? "alternate" : "normal",
    loop: direction === "both",
    easing: "easeInOutSine",
  });
}

// Progress bar animation
export function animateProgressBar(
  selector: string | Element | NodeListOf<Element>,
  percentage: number,
  options?: {
    duration?: number;
    easing?: string;
  }
) {
  const { duration = 800, easing = "easeOutQuad" } = options || {};

  return anime({
    targets: selector,
    width: [`0%`, `${percentage}%`],
    duration,
    easing,
  });
}

// Shake animation for errors
export function shake(
  selector: string | Element | NodeListOf<Element>,
  options?: {
    duration?: number;
  }
) {
  const { duration = 400 } = options || {};

  return anime({
    targets: selector,
    translateX: [-5, 5, -3, 3, -1, 1, 0],
    duration,
    easing: "easeInOutSine",
  });
}

// Layout shift animation (for reordering)
export function layoutShift(
  selector: string | Element | NodeListOf<Element>,
  options?: {
    duration?: number;
    easing?: string;
  }
) {
  const { duration = 300, easing = "easeOutCubic" } = options || {};

  return anime({
    targets: selector,
    opacity: [0.5, 1],
    scale: [0.98, 1],
    duration,
    easing,
  });
}

// Default export with all animations
export const AccountingAnimations = {
  fadeInCards,
  fadeOut,
  slideInFromLeft,
  slideInFromRight,
  scaleIn,
  scaleOut,
  countUp,
  animateTableRows,
  bounce,
  pulse,
  animateProgressBar,
  shake,
  layoutShift,
};

export default AccountingAnimations;