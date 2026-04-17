'use client';

interface ConfidenceBarProps {
  /** Confidence score between 0 and 1 */
  score: number;
  className?: string;
}

/**
 * Renders a horizontal confidence bar with graduated color and percentage label.
 * Used in scanner technologies, investigator fingerprints, and delivery insights.
 */
export default function ConfidenceBar({ score, className = '' }: ConfidenceBarProps) {
  const level = score >= 0.7 ? 'high' : score >= 0.4 ? 'medium' : 'low';

  return (
    <div className={`confidence-bar-container ${className}`}>
      <div className="confidence-bar">
        <div
          className={`confidence-bar-fill ${level}`}
          style={{ width: `${score * 100}%` }}
        />
      </div>
      <div className="confidence-value">{(score * 100).toFixed(0)}%</div>
    </div>
  );
}
