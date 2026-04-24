/**
 * SSE Verifier — intercepts SSE events during a scan and validates:
 *   - No dropped events
 *   - No duplicate renders
 *   - Final UI state matches backend output
 */
import { createHash } from 'crypto';
import type { Page } from '@playwright/test';
import type { SSEVerificationResult, SSEDuplicateRender, SSEEvent } from './types';

/**
 * Intercept SSE events by injecting a monitor into the page.
 * Call this BEFORE triggering the scan.
 */
export async function installSSEMonitor(page: Page): Promise<void> {
  await page.evaluate(() => {
    // Monkey-patch EventSource to capture SSE events
    const _OriginalEventSource = window.EventSource;
    const _capturedSSEEvents: Array<{
      timestamp: string;
      type: string;
      data: string;
      sequence: number;
    }> = [];
    let _sseSequence = 0;

    (window as unknown as Record<string, unknown>).__limma_sse_events = _capturedSSEEvents;
    (window as unknown as Record<string, unknown>).__limma_sse_done = false;

    class MonitoredEventSource extends _OriginalEventSource {
      constructor(url: string | URL, eventSourceInitDict?: EventSourceInit) {
        super(url, eventSourceInitDict);

        // Intercept onmessage
        const originalOnMessage = this.onmessage;
        this.addEventListener('message', (e: MessageEvent) => {
          _capturedSSEEvents.push({
            timestamp: new Date().toISOString(),
            type: 'message',
            data: e.data,
            sequence: _sseSequence++,
          });
        });

        // Intercept typed events
        const eventTypes = [
          'SCAN_STARTED', 'PAGE_CRAWLED', 'RISK_GENERATED', 'TECH_DETECTED',
          'HEADER_ANALYZED', 'SCAN_COMPLETED', 'CORRELATION_COMPLETED',
          'INFRA_SIGNAL_DETECTED', 'CMS_FINGERPRINT_MATCHED', 'DELIVERY_INSIGHT',
          'SECURITY_POSTURE', 'INVESTIGATION_COMPLETED',
        ];

        eventTypes.forEach((type) => {
          this.addEventListener(type, (e: MessageEvent) => {
            _capturedSSEEvents.push({
              timestamp: new Date().toISOString(),
              type,
              data: e.data,
              sequence: _sseSequence++,
            });
          });
        });

        // Monitor close
        this.addEventListener('error', () => {
          (window as unknown as Record<string, unknown>).__limma_sse_done = true;
        });
      }
    }

    // Replace global EventSource
    (window as unknown as Record<string, unknown>).EventSource = MonitoredEventSource as unknown as typeof EventSource;
  });
}

/**
 * Retrieve captured SSE events from the page.
 */
export async function getSSEEvents(page: Page): Promise<SSEEvent[]> {
  return page.evaluate(() => {
    const raw = (window as unknown as Record<string, unknown>).__limma_sse_events as Array<{
      timestamp: string;
      type: string;
      data: string;
      sequence: number;
    }>;
    return (raw || []).map((e) => ({
      timestamp: e.timestamp,
      type: e.type,
      data: (() => {
        try { return JSON.parse(e.data); } catch { return e.data; }
      })(),
      sequence: e.sequence,
    }));
  });
}

/**
 * Verify SSE consistency against the final backend data.
 */
export function verifySSEConsistency(
  sseEvents: SSEEvent[],
  backendFindingCount: number,
  frontendFindingCount: number,
): SSEVerificationResult {
  const errors: string[] = [];

  // 1. Extract unique event types
  const uniqueEventTypes = [...new Set(sseEvents.map((e) => e.type))];

  // 2. Check sequence for gaps (dropped events)
  let droppedEvents = 0;
  for (let i = 1; i < sseEvents.length; i++) {
    if (sseEvents[i].sequence !== sseEvents[i - 1].sequence + 1) {
      droppedEvents += sseEvents[i].sequence - sseEvents[i - 1].sequence - 1;
      errors.push(
        `Sequence gap detected: expected ${sseEvents[i - 1].sequence + 1}, got ${sseEvents[i].sequence}`,
      );
    }
  }

  // 3. Detect duplicate renders (same event type + same data hash appearing twice)
  const eventHashes = new Map<string, number>();
  for (const evt of sseEvents) {
    const hash = createHash('md5')
      .update(`${evt.type}::${JSON.stringify(evt.data)}`)
      .digest('hex');
    eventHashes.set(hash, (eventHashes.get(hash) || 0) + 1);
  }

  const duplicateRenders: SSEDuplicateRender[] = [];
  for (const [hash, count] of eventHashes) {
    if (count > 1) {
      const sample = sseEvents.find(
        (e) =>
          createHash('md5')
            .update(`${e.type}::${JSON.stringify(e.data)}`)
            .digest('hex') === hash,
      );
      duplicateRenders.push({
        event_type: sample?.type || 'unknown',
        data_hash: hash,
        occurrences: count,
      });
    }
  }

  if (duplicateRenders.length > 0) {
    errors.push(
      `Found ${duplicateRenders.length} duplicate SSE event render(s)`,
    );
  }

  // 4. Verify final state: UI finding count should match backend
  const finalStateMatches = frontendFindingCount === backendFindingCount;
  if (!finalStateMatches) {
    errors.push(
      `Final state mismatch: backend=${backendFindingCount}, frontend=${frontendFindingCount}`,
    );
  }

  // 5. Check event sequence validity
  const eventSequenceValid = droppedEvents === 0;

  return {
    total_events_received: sseEvents.length,
    unique_event_types: uniqueEventTypes,
    dropped_events: droppedEvents,
    duplicate_renders: duplicateRenders,
    final_state_matches_backend: finalStateMatches,
    event_sequence_valid: eventSequenceValid,
    errors,
  };
}
