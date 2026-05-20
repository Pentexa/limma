export interface EventSourceOptions {
  /** URL to connect to */
  url: string;
  /** Callback for incoming messages */
  onMessage: (event: MessageEvent) => void;
  /** Callback for errors */
  onError?: (error: Event) => void;
  /** Callback when connection opens */
  onOpen?: (event: Event) => void;
  /** Whether to reconnect on error */
  reconnect?: boolean;
  /** Reconnection delay in ms */
  reconnectDelay?: number;
  /** Maximum reconnection attempts */
  maxRetries?: number;
}

/**
 * Create a managed EventSource connection with reconnection logic.
 * Returns a cleanup function.
 */
export function createEventSource(options: EventSourceOptions): () => void {
  const {
    url,
    onMessage,
    onError,
    onOpen,
    reconnect = true,
    reconnectDelay = 3000,
    maxRetries = 5,
  } = options;

  let eventSource: EventSource | null = null;
  let retryCount = 0;
  let retryTimeout: ReturnType<typeof setTimeout> | null = null;
  let isCleanedUp = false;

  function connect() {
    if (isCleanedUp) return;

    eventSource = new EventSource(url);

    eventSource.onopen = (event) => {
      retryCount = 0;
      onOpen?.(event);
    };

    // Standard message
    eventSource.onmessage = onMessage;

    // Listen for custom backend events
    const eventTypes = [
      'SCAN_STARTED', 'CRAWLING_PAGE', 'PAGE_CRAWLED', 'RISK_GENERATED', 'TECH_DETECTED',
      'HEADER_ANALYZED', 'SCAN_COMPLETED', 'CRAWL_COMPLETE', 'CORRELATION_STARTED', 'CORRELATION_COMPLETE',
      'INFRA_SIGNAL_DETECTED', 'CMS_FINGERPRINT_MATCHED', 'DELIVERY_INSIGHT',
      'SECURITY_POSTURE', 'INVESTIGATION_COMPLETED', 'FINAL_RESULT'
    ];

    eventTypes.forEach(type => {
      eventSource?.addEventListener(type, (e) => onMessage(e as MessageEvent));
    });

    eventSource.onerror = (event) => {
      eventSource?.close();

      if (reconnect && !isCleanedUp && retryCount < maxRetries) {
        retryCount++;
        // Fixed delay (not exponential) to avoid long waits.
        // Only report error on final retry to prevent status flicker.
        retryTimeout = setTimeout(connect, reconnectDelay);
      } else if (!isCleanedUp) {
        // Final failure — now report the error
        onError?.(event);
      }
    };
  }

  connect();

  return () => {
    isCleanedUp = true;
    eventSource?.close();
    if (retryTimeout) clearTimeout(retryTimeout);
  };
}
