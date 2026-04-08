"use client";

import { CheckCircle2, HelpCircle, AlertTriangle, XCircle } from "lucide-react";

interface CertaintyNote {
  level: "certain" | "likely" | "uncertain" | "unknown";
  reason: string;
}

const certConfig: Record<string, { icon: React.ReactNode; color: string; bg: string; border: string; label: string }> = {
  certain: {
    icon: <CheckCircle2 className="h-3.5 w-3.5" />,
    color: "text-emerald-400",
    bg: "bg-emerald-500/10",
    border: "border-emerald-500/20",
    label: "Doğrulanmış",
  },
  likely: {
    icon: <HelpCircle className="h-3.5 w-3.5" />,
    color: "text-yellow-400",
    bg: "bg-yellow-500/10",
    border: "border-yellow-500/20",
    label: "Olası",
  },
  uncertain: {
    icon: <AlertTriangle className="h-3.5 w-3.5" />,
    color: "text-orange-400",
    bg: "bg-orange-500/10",
    border: "border-orange-500/20",
    label: "Belirsiz",
  },
  unknown: {
    icon: <XCircle className="h-3.5 w-3.5" />,
    color: "text-red-400",
    bg: "bg-red-500/10",
    border: "border-red-500/20",
    label: "Güvenilir Değil",
  },
};

export function CertaintyBadge({ certainty, compact }: { certainty?: CertaintyNote | null; compact?: boolean }) {
  if (!certainty) return null;
  const cfg = certConfig[certainty.level] || certConfig.unknown;

  if (compact) {
    return (
      <span className={`inline-flex items-center gap-1 px-1.5 py-0.5 rounded text-[9px] font-bold uppercase tracking-wide border ${cfg.bg} ${cfg.color} ${cfg.border}`} title={certainty.reason}>
        {cfg.icon}
        {cfg.label}
      </span>
    );
  }

  return (
    <div className={`flex items-start gap-2.5 px-4 py-3 rounded-xl border ${cfg.bg} ${cfg.border}`}>
      <div className={`mt-0.5 ${cfg.color}`}>{cfg.icon}</div>
      <div>
        <span className={`text-[11px] font-bold uppercase tracking-wider ${cfg.color}`}>{cfg.label}</span>
        <p className="text-[11px] text-gray-400 mt-0.5 leading-relaxed">{certainty.reason}</p>
      </div>
    </div>
  );
}

export function CertaintyInlineBadge({ level }: { level?: string | null }) {
  if (!level) return null;
  const cfg = certConfig[level] || certConfig.unknown;
  return (
    <span className={`inline-flex items-center gap-0.5 text-[8px] font-bold uppercase ${cfg.color}`} title={`Certainty: ${cfg.label}`}>
      {cfg.icon}
    </span>
  );
}
