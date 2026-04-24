import { useEffect, useCallback, useRef } from 'react';
import { useSSEStream } from './useSSEStream';
import { useScanSessionStore } from './scanSessionStore';
import type { StreamEvent } from './scanSessionStore';

// ── Hook Configuration ──
export interface UsePersistentSSEStreamOptions<T> {
  /** Unique module identifier: 'scanner' | 'investigator' | 'api-discovery' | ... */
  moduleId: string;
  /** SSE endpoint path, e.g. '/analyze/stream' */
  streamEndpoint: string;
  /** REST endpoint that returns the final result */
  fetchResult: (url: string) => Promise<T>;
}

// ── Hook Return Type ──
export interface UsePersistentSSEStreamReturn<T> {
  result: T | null;
  loading: boolean;
  error: string | null;
  events: Array<{ timestamp: string; type: string; message: string; raw?: unknown }>;
  streaming: boolean;
  execute: (url: string) => Promise<void>;
  reset: () => void;
  /** True if result was restored from a previous session (not live-streamed) */
  isRestored: boolean;
  /** Current session ID */
  sessionId: string | undefined;
}

/**
 * Persistent SSE Stream Hook
 *
 * Wraps useSSEStream and synchronizes all state changes to the global
 * ScanSessionStore (Zustand + localStorage). This ensures:
 *
 * 1. Module results persist across page navigations
 * 2. Results survive page refresh (via localStorage)
 * 3. Cross-module correlation is possible via shared session
 *
 * Usage:
 * ```tsx
 * const { result, loading, error, events, execute, isRestored } =
 *   usePersistentSSEStream<WebScanResult>({
 *     moduleId: 'scanner',
 *     streamEndpoint: '/analyze/stream',
 *     fetchResult: analyzeSite,
 *   });
 * ```
 */
export function usePersistentSSEStream<T>(
  options: UsePersistentSSEStreamOptions<T>,
): UsePersistentSSEStreamReturn<T> {
  const { moduleId, streamEndpoint, fetchResult } = options;

  const store = useScanSessionStore();
  const session = store.activeSession;
  const sessionRef = useRef(session);
  sessionRef.current = session;

  // Get persisted state from store (if any)
  const persistedResult = session
    ? store.getModuleResult(session.id, moduleId)
    : undefined;

  // Delegate to the original SSE hook
  const {
    result: streamResult,
    loading,
    error,
    events,
    streaming,
    execute: rawExecute,
    reset: rawReset,
  } = useSSEStream<T>({
    streamEndpoint,
    fetchResult,
  });

  // ── Sync streaming status to store ──
  useEffect(() => {
    const s = sessionRef.current;
    if (!s || !moduleId) return;

    if (streaming) {
      store.setStreamState(s.id, moduleId, { status: 'streaming', startTime: Date.now() });
      store.setModuleLoading(s.id, moduleId);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [streaming, moduleId]);

  // ── Sync stream events to store ──
  useEffect(() => {
    const s = sessionRef.current;
    if (!s || !moduleId || events.length === 0) return;

    const lastEvent = events[events.length - 1];
    const storeEvent: StreamEvent = {
      timestamp: lastEvent.timestamp,
      type: lastEvent.type,
      message: lastEvent.message,
      raw: lastEvent.raw,
    };
    store.appendStreamEvent(s.id, moduleId, storeEvent);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [events.length, moduleId]);

  // ── Sync final result to store ──
  useEffect(() => {
    const s = sessionRef.current;
    if (!s || !moduleId || !streamResult) return;

    store.setModuleResult(s.id, moduleId, {
      moduleId,
      moduleName: moduleId,
      targetUrl: s.targetUrl,
      result: streamResult,
      status: 'success',
    });
    store.setStreamState(s.id, moduleId, {
      status: 'completed',
      result: streamResult,
      endTime: Date.now(),
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [streamResult, moduleId]);

  // ── Sync errors to store ──
  useEffect(() => {
    const s = sessionRef.current;
    if (!s || !moduleId || !error) return;

    store.setModuleError(s.id, moduleId, error);
    store.setStreamState(s.id, moduleId, {
      status: 'error',
      error,
      endTime: Date.now(),
    });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [error, moduleId]);

  // ── Execute with session management ──
  const execute = useCallback(
    async (url: string) => {
      // Create session if none exists, or if target URL changed
      const current = useScanSessionStore.getState().activeSession;
      if (!current || current.targetUrl !== url) {
        if (current) {
          store.closeSession(current.id);
        }
        store.createSession(url);
      }
      return rawExecute(url);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [rawExecute],
  );

  // ── Reset ──
  const reset = useCallback(() => {
    rawReset();
  }, [rawReset]);

  // Compute whether the displayed result came from the store (restored)
  const isRestored = !streaming && !loading && !streamResult && !!persistedResult?.result;

  // Use live result if available, else fall back to persisted
  const effectiveResult = (streamResult || persistedResult?.result || null) as T | null;

  return {
    result: effectiveResult,
    loading,
    error: error || (persistedResult?.status === 'error' ? persistedResult.error || null : null),
    events,
    streaming,
    execute,
    reset,
    isRestored,
    sessionId: session?.id,
  };
}
