'use client';

import type { ScanEvent } from '@/lib/api';

interface ScanTimelineProps {
  events: ScanEvent[];
}

/**
 * Renders a vertical timeline of scan events with color-coded severity dots.
 * Extracted from the Scanner page's timeline tab.
 */
export default function ScanTimeline({ events }: ScanTimelineProps) {
  if (events.length === 0) return null;

  return (
    <div className="glass-card">
      <div className="timeline">
        {events.map((evt, i) => (
          <div key={i} className="timeline-item">
            <div
              className={`timeline-dot ${
                evt.level === 'WARN' ? 'warn' : evt.level === 'ERROR' ? 'error' : ''
              }`}
            />
            <div className="timeline-type">{evt.event_type}</div>
            <div className="timeline-message">{evt.message}</div>
            <div className="timeline-time">
              {new Date(evt.timestamp).toLocaleTimeString()}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
