import { useState, useRef, useCallback } from 'react';
import { apiStream } from '@/lib/api';

// ── SSE Stream Event (normalized for UI consumption) ──
export interface SSEEvent {
  timestamp: string;
  type: string;
  message: string;
  raw?: unknown;
}

// ── Hook Configuration ──
export interface UseSSEStreamOptions<T> {
  /** SSE endpoint path, e.g. '/analyze/stream' */
  streamEndpoint: string;
  /** REST endpoint that returns the final result */
  fetchResult: (url: string) => Promise<T>;
  /** Format incoming SSE events into a display-friendly message (optional) */
  formatEvent?: (evt: { type: string; data: unknown }) => string;
}

// ── Hook Return Type ──
export interface UseSSEStreamReturn<T> {
  result: T | null;
  loading: boolean;
  error: string | null;
  events: SSEEvent[];
  streaming: boolean;
  execute: (url: string) => Promise<void>;
  reset: () => void;
}

// ── Default event formatter ──
function defaultFormatEvent(evt: { type: string; data: unknown }): string {
  if (typeof evt.data === 'object' && evt.data !== null) {
    const obj = evt.data as Record<string, unknown>;
    return obj.message ? String(obj.message) : JSON.stringify(evt.data);
  }
  return String(evt.data);
}

/**
 * Custom hook that centralizes SSE (Server-Sent Events) streaming + REST fetch pattern.
 *
 * Eliminates boilerplate duplication across Scanner, Investigator, and any future
 * pages that combine an SSE progress stream with a final REST API call.
 *
 * Usage:
 * ```tsx
 * const { result, loading, error, events, streaming, execute } = useSSEStream({
 *   streamEndpoint: '/analyze/stream',
 *   fetchResult: (url) => analyzeSite(url),
 * });
 * ```
 */
export function useSSEStream<T>(options: UseSSEStreamOptions<T>): UseSSEStreamReturn<T> {
  const { streamEndpoint, fetchResult, formatEvent = defaultFormatEvent } = options;

  const [result, setResult] = useState<T | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [events, setEvents] = useState<SSEEvent[]>([]);
  const [streaming, setStreaming] = useState(false);
  const closeRef = useRef<(() => void) | null>(null);

  const cleanup = useCallback(() => {
    if (closeRef.current) {
      closeRef.current();
      closeRef.current = null;
    }
    setStreaming(false);
  }, []);

  const reset = useCallback(() => {
    cleanup();
    setResult(null);
    setLoading(false);
    setError(null);
    setEvents([]);
  }, [cleanup]);

  const execute = useCallback(async (url: string) => {
    // Reset state
    setLoading(true);
    setError(null);
    setResult(null);
    setEvents([]);
    setStreaming(true);

    // Start SSE stream for real-time events
    closeRef.current = apiStream(
      streamEndpoint,
      { url },
      (evt) => {
        const event: SSEEvent = {
          timestamp: new Date().toISOString(),
          type: evt.type,
          message: formatEvent(evt),
          raw: evt.data,
        };
        setEvents(prev => [...prev, event]);
      },
      () => setStreaming(false),
      () => setStreaming(false),
    );

    // Fetch final result via REST
    try {
      const res = await fetchResult(url);
      setResult(res);
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Operation failed');
    } finally {
      setLoading(false);
      cleanup();
    }
  }, [streamEndpoint, fetchResult, formatEvent, cleanup]);

  return { result, loading, error, events, streaming, execute, reset };
}
