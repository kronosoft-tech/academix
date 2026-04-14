/**
 * CSS-based Animations
 * Simple, reliable animations using CSS transitions
 */

// Entrance animation classes
export const entranceAnimations = {
  fade: "animate-fade-in",
  scale: "animate-scale-in", 
  slideUp: "animate-slide-up",
  slideDown: "animate-slide-down",
  slideLeft: "animate-slide-left",
  slideRight: "animate-slide-right",
};

// Stagger delay for lists
export const staggerDelay = (index: number, baseDelay = 50) => ({
  animationDelay: `${index * baseDelay}ms`,
});

// Modal animations
export const modalEntrance = "animate-modal-in";
export const modalExit = "animate-modal-out";

// Page transitions
export const pageTransition = "animate-page-in";

export default entranceAnimations;