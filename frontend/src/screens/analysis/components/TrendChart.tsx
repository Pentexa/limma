"use client";

import { useState, useMemo } from "react";
import { cn } from "@/shared/lib/utils";
import type { ApiTrendPoint } from "@/shared/types/api";
import { TrendingUp } from "lucide-react";

interface TrendChartProps {
  points: ApiTrendPoint[];
  className?: string;
}

// Simple cubic bezier curve generator for ultra-smooth lines
function smoothLine(points: [number, number][]): string {
  if (points.length === 0) return "";
  if (points.length === 1) return `M${points[0][0]},${points[0][1]}`;
  let d = `M${points[0][0]},${points[0][1]}`;
  for (let i = 0; i < points.length - 1; i++) {
    const p0 = points[i];
    const p1 = points[i + 1];
    // Tension factor of 0.4 creates natural smooth curves without overshooting
    const cp1x = p0[0] + (p1[0] - p0[0]) * 0.4;
    const cp1y = p0[1];
    const cp2x = p1[0] - (p1[0] - p0[0]) * 0.4;
    const cp2y = p1[1];
    d += ` C${cp1x},${cp1y} ${cp2x},${cp2y} ${p1[0]},${p1[1]}`;
  }
  return d;
}

export function TrendChart({ points, className }: TrendChartProps) {
  const [hoverIndex, setHoverIndex] = useState<number | null>(null);

  if (points.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-16 bg-[#080808] border border-white/[0.05] rounded-xl text-muted-foreground/50">
        <div className="w-16 h-16 rounded-full bg-white/[0.02] flex items-center justify-center mb-4">
          <TrendingUp className="h-6 w-6 opacity-30" />
        </div>
        <span className="text-[12px] font-mono uppercase tracking-widest font-bold">No Trend Data</span>
      </div>
    );
  }

  if (points.length === 1) {
    const p = points[0];
    const scoreColor = p.score >= 80 ? "text-verified" : p.score >= 50 ? "text-attention" : "text-risk";
    return (
      <div className="flex flex-col md:flex-row items-center justify-center gap-12 py-12 bg-[#080808] border border-white/[0.05] rounded-xl relative overflow-hidden">
        <div className="absolute top-0 left-0 w-full h-full bg-gradient-to-b from-primary/5 to-transparent pointer-events-none" />
        
        <div className="flex flex-col items-center z-10">
          <span className="text-[10px] uppercase tracking-widest text-muted-foreground/60 font-bold mb-3">Initial Security Score</span>
          <div className={cn("text-[80px] font-bold leading-none font-mono drop-shadow-[0_0_30px_currentColor]", scoreColor)}>
            {p.score}
          </div>
          <span className="text-[10px] text-primary mt-3 uppercase tracking-widest font-bold px-3 py-1 bg-primary/10 rounded-full border border-primary/20">
            Baseline Established
          </span>
        </div>
        
        <div className="h-24 w-px bg-white/[0.05] hidden md:block z-10" />
        
        <div className="flex flex-col items-center z-10">
          <span className="text-[10px] uppercase tracking-widest text-muted-foreground/60 font-bold mb-3">Total Findings</span>
          <div className="text-[64px] font-bold leading-none font-mono text-attention drop-shadow-[0_0_20px_rgba(var(--attention),0.3)]">
            {p.total_findings}
          </div>
        </div>
      </div>
    );
  }

  // Memoize heavy calculations
  const { sorted, W, H, PAD, plotW, plotH, xScale, yScore, yFindings, scoreAreaPath, scorePath, findingsPath } = useMemo(() => {
    const sorted = [...points].sort((a, b) => a.timestamp_sec - b.timestamp_sec);
    const maxScore = 100;
    const maxFindings = Math.max(...sorted.map(p => p.total_findings), 10);
  
    const W = 800;
    const H = 280;
    const PAD = { top: 30, right: 30, bottom: 40, left: 50 };
    const plotW = W - PAD.left - PAD.right;
    const plotH = H - PAD.top - PAD.bottom;
  
    const xScale = (i: number) => PAD.left + (i / (sorted.length - 1)) * plotW;
    const yScore = (v: number) => PAD.top + plotH - (v / maxScore) * plotH;
    const yFindings = (v: number) => PAD.top + plotH - (v / maxFindings) * plotH;
  
    const scoreCoords: [number, number][] = sorted.map((p, i) => [xScale(i), yScore(p.score)]);
    const findingsCoords: [number, number][] = sorted.map((p, i) => [xScale(i), yFindings(p.total_findings)]);
    
    const scorePath = smoothLine(scoreCoords);
    const findingsPath = smoothLine(findingsCoords);
    
    const scoreAreaPath = `${scorePath} L${xScale(sorted.length - 1)},${H - PAD.bottom} L${PAD.left},${H - PAD.bottom} Z`;

    return { sorted, W, H, PAD, plotW, plotH, xScale, yScore, yFindings, scoreAreaPath, scorePath, findingsPath };
  }, [points]);

  // Handle interactive hover
  const handleMouseMove = (e: React.MouseEvent<SVGSVGElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const viewPortX = (x / rect.width) * W;
    
    // Find closest point index
    let closestIndex = 0;
    let minDiff = Infinity;
    for (let i = 0; i < sorted.length; i++) {
      const diff = Math.abs(xScale(i) - viewPortX);
      if (diff < minDiff) {
        minDiff = diff;
        closestIndex = i;
      }
    }
    
    // Only show tooltip if mouse is reasonably close to the chart area
    if (viewPortX >= PAD.left - 20 && viewPortX <= W - PAD.right + 20) {
      setHoverIndex(closestIndex);
    } else {
      setHoverIndex(null);
    }
  };

  return (
    <div className={cn("w-full relative bg-[#060606] border border-white/[0.05] rounded-xl p-4 shadow-2xl", className)}>
      <svg 
        viewBox={`0 0 ${W} ${H}`} 
        className="w-full h-auto overflow-visible"
        onMouseMove={handleMouseMove}
        onMouseLeave={() => setHoverIndex(null)}
      >
        <defs>
          <linearGradient id="scoreGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="hsl(var(--primary))" stopOpacity="0.4" />
            <stop offset="50%" stopColor="hsl(var(--primary))" stopOpacity="0.1" />
            <stop offset="100%" stopColor="hsl(var(--primary))" stopOpacity="0.0" />
          </linearGradient>
          <linearGradient id="findingsGradient" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="hsl(var(--attention))" stopOpacity="0.15" />
            <stop offset="100%" stopColor="hsl(var(--attention))" stopOpacity="0.0" />
          </linearGradient>
          <filter id="glow" x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur stdDeviation="4" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
        </defs>

        {/* Subtle Background Grid */}
        {[0, 25, 50, 75, 100].map(v => (
          <g key={`grid-${v}`}>
            <line 
              x1={PAD.left} y1={yScore(v)} 
              x2={W - PAD.right} y2={yScore(v)} 
              stroke="white" 
              className="opacity-5" 
              strokeWidth="1" 
            />
            <text 
              x={PAD.left - 15} y={yScore(v) + 3} 
              textAnchor="end" 
              className="fill-muted-foreground/40 text-[9px] font-mono font-bold"
            >
              {v}
            </text>
          </g>
        ))}

        {/* Areas */}
        <path d={scoreAreaPath} fill="url(#scoreGradient)" className="pointer-events-none" />

        {/* Lines */}
        <path d={findingsPath} fill="none" stroke="hsl(var(--attention))" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" className="opacity-40 pointer-events-none" />
        <path d={scorePath} fill="none" stroke="hsl(var(--primary))" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" filter="url(#glow)" className="pointer-events-none" />

        {/* Bottom Axis Labels */}
        {sorted.map((p, i) => {
          // Show fewer labels if too many points
          if (sorted.length > 10 && i % Math.ceil(sorted.length / 8) !== 0 && i !== sorted.length - 1) return null;
          const x = xScale(i);
          const date = new Date(p.timestamp_sec * 1000);
          return (
            <text 
              key={`label-${p.scan_id}`} 
              x={x} y={H - 10} 
              textAnchor="middle" 
              className="fill-muted-foreground/40 text-[9px] font-mono uppercase tracking-widest font-bold pointer-events-none"
            >
              {date.toLocaleDateString("en", { month: "short", day: "numeric" })}
            </text>
          );
        })}

        {/* Interactive Hover Tooltip & Crosshair */}
        {hoverIndex !== null && (
          <g className="pointer-events-none transition-all duration-200 ease-out">
            {/* Vertical Guide Line */}
            <line 
              x1={xScale(hoverIndex)} y1={PAD.top} 
              x2={xScale(hoverIndex)} y2={H - PAD.bottom} 
              stroke="white" strokeWidth="1" className="opacity-20" strokeDasharray="4,4" 
            />
            
            {/* Hover Points */}
            <circle cx={xScale(hoverIndex)} cy={yScore(sorted[hoverIndex].score)} r="6" fill="hsl(var(--primary))" className="opacity-30" />
            <circle cx={xScale(hoverIndex)} cy={yScore(sorted[hoverIndex].score)} r="3" fill="white" filter="url(#glow)" />
            
            <circle cx={xScale(hoverIndex)} cy={yFindings(sorted[hoverIndex].total_findings)} r="5" fill="hsl(var(--attention))" className="opacity-20" />
            <circle cx={xScale(hoverIndex)} cy={yFindings(sorted[hoverIndex].total_findings)} r="2" fill="hsl(var(--attention))" />

            {/* Tooltip Card */}
            <g transform={`translate(${
              xScale(hoverIndex) > W - 150 ? xScale(hoverIndex) - 130 : xScale(hoverIndex) + 15
            }, ${PAD.top})`}>
              <rect x="0" y="0" width="115" height="65" rx="6" fill="black" className="opacity-90" stroke="rgba(255,255,255,0.1)" strokeWidth="1" />
              <text x="12" y="20" fill="white" className="text-[9px] font-mono font-bold opacity-60 uppercase tracking-wider">
                {new Date(sorted[hoverIndex].timestamp_sec * 1000).toLocaleDateString("en", { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}
              </text>
              <text x="12" y="38" fill="hsl(var(--primary))" className="text-[12px] font-mono font-bold">
                Score: {sorted[hoverIndex].score}
              </text>
              <text x="12" y="52" fill="hsl(var(--attention))" className="text-[10px] font-mono font-bold">
                Findings: {sorted[hoverIndex].total_findings}
              </text>
            </g>
          </g>
        )}
      </svg>

      {/* Legend */}
      <div className="absolute top-6 right-6 flex items-center gap-6 text-[10px] text-muted-foreground font-bold uppercase tracking-widest bg-black/40 px-4 py-2 rounded-full border border-white/[0.05] backdrop-blur-md">
        <span className="flex items-center gap-2">
          <span className="w-3 h-3 rounded-full border-2 border-primary bg-primary/20 shadow-[0_0_10px_rgba(var(--primary),0.5)]" /> 
          Score
        </span>
        <span className="flex items-center gap-2">
          <span className="w-3 h-3 rounded-full border-2 border-attention bg-attention/20" /> 
          Findings
        </span>
      </div>
    </div>
  );
}
