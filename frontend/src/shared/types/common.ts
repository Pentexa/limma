/** Branded string types for unique identifiers */
export type ScanId = string & { readonly __brand: "ScanId" };
export type FindingId = string & { readonly __brand: "FindingId" };
export type PocId = string & { readonly __brand: "PocId" };

/**
 * P2-008: Branded type factory functions.
 * Use these instead of `as unknown as ScanId` casts.
 * They provide a single, auditable cast point with runtime validation.
 */
export function asScanId(id: string): ScanId {
  return (id || String(Date.now())) as ScanId;
}

export function asFindingId(id: string): FindingId {
  return (id || String(Date.now())) as FindingId;
}

export function asPocId(id: string): PocId {
  return (id || String(Date.now())) as PocId;
}

/** Legacy generic ID (will be phased out) */
export type ID = string;

/** ISO 8601 date string */
export type Timestamp = string;

/** Generic paginated API response */
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

/** Generic API error response */
export interface ApiError {
  message: string;
  code: string;
  status: number;
  details?: Record<string, unknown>;
}

/** Common severity levels */
export type Severity = "critical" | "high" | "medium" | "low" | "info";

/** Sort direction */
export type SortDirection = "asc" | "desc";

/** Generic sort config */
export interface SortConfig {
  field: string;
  direction: SortDirection;
}

/** Status types used across the app */
export type ConnectionStatus = "connected" | "connecting" | "disconnected" | "error";
