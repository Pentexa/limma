'use client';

import { getScoreColor } from '@/lib/api';

interface ScoreGaugeProps {
  score: number;
  size?: number;
  label?: string;
}

export default function ScoreGauge({ score, size = 140, label = 'Health Score' }: ScoreGaugeProps) {
  const radius = (size - 20) / 2;
  const circumference = 2 * Math.PI * radius;
  const progress = (score / 100) * circumference;
  const color = getScoreColor(score);
  const center = size / 2;

  return (
    <div className="score-gauge" style={{ width: size, height: size }}>
      <svg width={size} height={size}>
        {/* Background ring */}
        <circle
          cx={center}
          cy={center}
          r={radius}
          fill="none"
          stroke="rgba(100, 116, 139, 0.08)"
          strokeWidth="10"
        />
        {/* Subtle secondary trace */}
        <circle
          cx={center}
          cy={center}
          r={radius}
          fill="none"
          stroke="rgba(100, 116, 139, 0.04)"
          strokeWidth="10"
          strokeDasharray="4 8"
        />
        {/* Active ring with gradient */}
        <defs>
          <linearGradient id={`gaugeGrad-${score}`} x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor={color} />
            <stop offset="100%" stopColor={color} stopOpacity="0.5" />
          </linearGradient>
        </defs>
        <circle
          cx={center}
          cy={center}
          r={radius}
          fill="none"
          stroke={`url(#gaugeGrad-${score})`}
          strokeWidth="10"
          strokeDasharray={circumference}
          strokeDashoffset={circumference - progress}
          strokeLinecap="round"
          style={{
            transition: 'stroke-dashoffset 1.2s cubic-bezier(0.4, 0, 0.2, 1)',
            filter: `drop-shadow(0 0 10px ${color}80)`,
          }}
        />
        {/* Glowing tip dot */}
        {score > 0 && (
          <circle
            cx={center + radius * Math.cos(((score / 100) * 360 - 90) * Math.PI / 180)}
            cy={center + radius * Math.sin(((score / 100) * 360 - 90) * Math.PI / 180)}
            r="5"
            fill={color}
            style={{ filter: `drop-shadow(0 0 8px ${color})` }}
          />
        )}
      </svg>
      <div className="score-gauge-inner">
        <div className="score-gauge-value" style={{ color }}>{score}</div>
        <div className="score-gauge-label">{label}</div>
      </div>
    </div>
  );
}
