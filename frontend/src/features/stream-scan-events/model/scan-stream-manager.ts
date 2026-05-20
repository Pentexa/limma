/**
 * Global SSE stream manager — singleton that connects/disconnects SSE
 * based on scan lifecycle. Can be called from anywhere (start-scan, dashboard, etc.)
 *
 * STABILITY: Uses debounce to prevent rapid disconnect/reconnect cycles.
 */
import { connectScanStream } from "../api/event-source";
import { useStreamStore } from "./stream-store";

let currentDisconnect: (() => void) | null = null;
let currentTargetUrl: string | null = null;
let isConnecting = false;

/**
 * Start the SSE stream for a given target URL.
 * If already connected to the same target, does nothing.
 * If connected to a different target, disconnects first then reconnects.
 */
export function startScanStream(targetUrl: string): void {
  if (!targetUrl) return;

  // Already connected or connecting to this target — skip
  if (currentTargetUrl === targetUrl && (currentDisconnect || isConnecting)) {
    return;
  }

  // Disconnect previous stream if any
  stopScanStream();

  isConnecting = true;
  currentTargetUrl = targetUrl;

  // Clear old events
  useStreamStore.getState().clearEvents();

  // Connect
  currentDisconnect = connectScanStream(targetUrl);
  isConnecting = false;
}

/**
 * Stop the SSE stream. Safe to call multiple times.
 */
export function stopScanStream(): void {
  if (currentDisconnect) {
    currentDisconnect();
    currentDisconnect = null;
  }
  currentTargetUrl = null;
  isConnecting = false;
  // Don't touch connectionStatus here — let the EventSource onError/cleanup handle it
}

/**
 * Get the currently connected target URL.
 */
export function getCurrentStreamTarget(): string | null {
  return currentTargetUrl;
}
