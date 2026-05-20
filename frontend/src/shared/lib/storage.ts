/**
 * Browser localStorage persistence for user preferences.
 * P2-011: Provides a type-safe wrapper around localStorage with
 * JSON serialization and fallback defaults for SSR safety.
 */

const STORAGE_PREFIX = "limma:";

/** Get a persisted value, or return the fallback if not found / SSR. */
export function getPersistedValue<T>(key: string, fallback: T): T {
  if (typeof window === "undefined") return fallback;
  try {
    const raw = localStorage.getItem(`${STORAGE_PREFIX}${key}`);
    if (raw === null) return fallback;
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

/** Persist a value to localStorage. */
export function setPersistedValue<T>(key: string, value: T): void {
  if (typeof window === "undefined") return;
  try {
    localStorage.setItem(`${STORAGE_PREFIX}${key}`, JSON.stringify(value));
  } catch {
    // Storage full or disabled — silently fail
  }
}

/** Remove a persisted value. */
export function removePersistedValue(key: string): void {
  if (typeof window === "undefined") return;
  localStorage.removeItem(`${STORAGE_PREFIX}${key}`);
}

// ── Common storage keys ──

/** Last selected profile ID */
export const STORAGE_KEYS = {
  SELECTED_PROFILE: "selected-profile",
  SIDEBAR_COLLAPSED: "sidebar-collapsed",
  FILTER_PREFERENCES: "filter-preferences",
} as const;
