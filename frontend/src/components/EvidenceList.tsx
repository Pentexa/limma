'use client';

interface EvidenceListProps {
  /** Simple evidence strings (bullet list) */
  items?: string[];
  /** Evidence with type label and content */
  labeled?: { type: string; content: string }[];
}

/**
 * Renders evidence items in one of two styles:
 * - Bullet list: cyan-colored mono text (used for fingerprint evidences, consistency evidences)
 * - Labeled: evidence-item boxes with type + content (used for risk insights, delivery, security)
 */
export default function EvidenceList({ items, labeled }: EvidenceListProps) {
  if (labeled && labeled.length > 0) {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
        {labeled.map((e, i) => (
          <div key={i} className="evidence-item">
            <div className="evidence-type">{e.type}</div>
            <div className="evidence-content">{e.content}</div>
          </div>
        ))}
      </div>
    );
  }

  if (items && items.length > 0) {
    return (
      <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
        {items.map((e, i) => (
          <div
            key={i}
            className="text-xs"
            style={{ color: 'var(--accent-cyan)', fontFamily: 'var(--font-mono)' }}
          >
            • {e}
          </div>
        ))}
      </div>
    );
  }

  return null;
}
