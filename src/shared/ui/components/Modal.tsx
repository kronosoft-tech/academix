import type { ReactNode } from "react";
import { useRef, useState } from "react";
import { cn } from "../../../lib/utils";
import { useModalAnimation } from "../../../lib/animations/hooks";

interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title?: string;
  children: ReactNode;
  size?: "sm" | "md" | "lg" | "xl";
  /** Disable animations */
  animate?: boolean;
}

export function Modal({ isOpen, onClose, title, children, size = "md", animate = true }: ModalProps) {
  const backdropRef = useRef<HTMLDivElement>(null);
  const modalRef = useRef<HTMLDivElement>(null);
  const [renderModal, setRenderModal] = useState(false);

  const { shouldRender, isExiting } = useModalAnimation({
    isOpen,
    backdropRef,
    modalRef,
    entranceType: "scale",
    onExitComplete: () => {
      setRenderModal(false);
    },
  });

  // Start rendering after first open (for entrance animation)
  if (isOpen && !renderModal && !isExiting) {
    setRenderModal(true);
  }

  // Don't render if we're not open and not exiting
  if (!shouldRender && !isOpen) return null;

  const sizes = {
    sm: "max-w-md",
    md: "max-w-lg",
    lg: "max-w-2xl",
    xl: "max-w-4xl",
  };

  // Initial styles - start visible and let anime.js animate to final state
  const getInitialStyles = (_isBackdrop: boolean): string => {
    if (!animate) return "";
    // Don't set initial opacity-0 - that causes the "white screen" issue
    // anime.js will handle the opacity animation
    return "";
  };

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      <div className="flex min-h-full items-center justify-center p-4">
        {/* Backdrop */}
        <div
          ref={backdropRef}
          className={cn(
            "fixed inset-0 bg-black/50 transition-opacity",
            getInitialStyles(true)
          )}
          onClick={onClose}
        />
        
        {/* Modal */}
        <div
          ref={modalRef}
          className={cn(
            "relative w-full bg-white rounded-lg shadow-xl",
            sizes[size],
            getInitialStyles(false)
          )}
          style={{ transformOrigin: "center" }}
        >
          {title && (
            <div className="flex items-center justify-between px-6 py-4 border-b">
              <h3 className="text-lg font-semibold text-gray-900">{title}</h3>
              <button
                onClick={onClose}
                className="text-gray-400 hover:text-gray-500 transition-colors"
              >
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
          )}
          <div className="px-6 py-4">{children}</div>
        </div>
      </div>
    </div>
  );
}