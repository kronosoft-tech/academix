/**
 * Lightweight app-wide event bus for cross-module data refresh.
 *
 * When a module mutates data that other modules might display,
 * it emits an event. Listening modules refetch their data.
 *
 * Usage:
 *   emit: appEvents.emit("payments:changed")
 *   listen: appEvents.on("payments:changed", callback)
 *   cleanup: appEvents.off("payments:changed", callback)
 */

type EventCallback = () => void;

class AppEventBus {
  private listeners: Map<string, Set<EventCallback>> = new Map();

  on(event: string, callback: EventCallback): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(callback);
  }

  off(event: string, callback: EventCallback): void {
    this.listeners.get(event)?.delete(callback);
  }

  emit(event: string): void {
    this.listeners.get(event)?.forEach((cb) => cb());
  }
}

export const appEvents = new AppEventBus();

// Event names as constants to avoid typos
export const APP_EVENTS = {
  PAYMENTS_CHANGED: "payments:changed",
  ACCOUNTING_CHANGED: "accounting:changed",
  STUDENTS_CHANGED: "students:changed",
  COURSES_CHANGED: "courses:changed",
  GROUPS_CHANGED: "groups:changed",
  ATTENDANCE_CHANGED: "attendance:changed",
} as const;
