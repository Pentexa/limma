/**
 * Generic snake_case → camelCase conversion utilities for API responses.
 */

/** Convert a snake_case string to camelCase */
export function toCamelCase(str: string): string {
  return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}

/** Convert a camelCase string to snake_case */
export function toSnakeCase(str: string): string {
  return str.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
}

/**
 * Recursively convert all keys of an object from snake_case to camelCase.
 * Handles nested objects and arrays.
 */
export function mapKeysToCamelCase<T>(obj: unknown): T {
  if (obj === null || obj === undefined) return obj as T;

  if (Array.isArray(obj)) {
    return obj.map((item) => mapKeysToCamelCase(item)) as T;
  }

  if (typeof obj === "object" && obj !== null) {
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      const camelKey = toCamelCase(key);
      result[camelKey] = mapKeysToCamelCase(value);
    }
    return result as T;
  }

  return obj as T;
}

/**
 * Recursively convert all keys of an object from camelCase to snake_case.
 * Used when sending data to the backend.
 */
export function mapKeysToSnakeCase<T>(obj: unknown): T {
  if (obj === null || obj === undefined) return obj as T;

  if (Array.isArray(obj)) {
    return obj.map((item) => mapKeysToSnakeCase(item)) as T;
  }

  if (typeof obj === "object" && obj !== null) {
    const result: Record<string, unknown> = {};
    for (const [key, value] of Object.entries(obj)) {
      const snakeKey = toSnakeCase(key);
      result[snakeKey] = mapKeysToSnakeCase(value);
    }
    return result as T;
  }

  return obj as T;
}
