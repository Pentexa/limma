import { SSE_STREAM_URL, SSE_RECONNECT_DELAY, SSE_MAX_RETRIES } from "@/shared/config/constants";
import { createEventSource, type EventSourceOptions } from "@/shared/lib/create-event-source";

export interface SSEMessage<T = unknown> {
  type: string;
  data: T;
  timestamp: string;
}

/** Parse an SSE message event into a typed structure */
export function parseSSEMessage<T>(event: MessageEvent): SSEMessage<T> {
  try {
    const parsed = JSON.parse(event.data);
    const type = parsed.event_type || parsed.type || event.type || "unknown";
    const data = parsed.message || parsed.data || parsed;
    return {
      type,
      data,
      timestamp: parsed.timestamp ?? new Date().toISOString(),
    };
  } catch {
    return {
      type: "raw",
      data: event.data as T,
      timestamp: new Date().toISOString(),
    };
  }
}

/** Create an SSE connection to the scan stream endpoint */
export function connectToScanStream(
  options: Omit<EventSourceOptions, "url" | "reconnectDelay" | "maxRetries"> & { targetUrl?: string }
): () => void {
  const streamUrl = options.targetUrl 
    ? `${SSE_STREAM_URL}?url=${encodeURIComponent(options.targetUrl)}`
    : SSE_STREAM_URL;

  return createEventSource({
    ...options,
    url: streamUrl,
    reconnectDelay: SSE_RECONNECT_DELAY,
    maxRetries: SSE_MAX_RETRIES,
  });
}

/** Create an SSE connection to any endpoint */
export function connectToSSE(
  url: string,
  options: Omit<EventSourceOptions, "url">
): () => void {
  return createEventSource({
    ...options,
    url,
  });
}
