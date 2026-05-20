/**
 * Shared config barrel export.
 * P2-007: Standardized import paths for shared configuration.
 */

export { SEVERITY_WEIGHT, compareBySeverity } from "./priority";
export {
  API_BASE_URL,
  SSE_STREAM_URL,
  SSE_RECONNECT_DELAY,
  SSE_MAX_RETRIES,
  SCAN_PHASES,
  PHASE_LABELS,
  type ScanPhase,
} from "./constants";
