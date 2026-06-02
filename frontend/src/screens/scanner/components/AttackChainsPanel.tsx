import { cn } from "@/shared/lib/utils";
import type { ApiNormalizedAuditReport, ApiCanonicalFinding } from "@/shared/types/api";
import { Link2, ShieldAlert, Target, ShieldQuestion, BrainCircuit, Info, Zap, AlertTriangle, ShieldCheck } from "lucide-react";

// --- VIEW MODEL ---

export interface AttackChainFindingVM {
  slug: string;
  title: string;
  severity: string;
}

export interface AttackChainViewModel {
  id: string;
  score: number;
  narrative: string;
  exploitability: "actionable" | "theoretical" | "inert";
  conditions: string[];
  context: string[];
  findings: AttackChainFindingVM[];
  verificationStatus?: string;
  verificationReasoning?: string;
}

// --- MAPPER ---

function mapApiToViewModel(report: ApiNormalizedAuditReport): AttackChainViewModel[] {
  if (!report.attack_paths || !Array.isArray(report.attack_paths)) return [];

  // Create a lookup for canonical findings
  const findingsMap = new Map<string, ApiCanonicalFinding>();
  if (report.canonical_findings && Array.isArray(report.canonical_findings)) {
    for (const f of report.canonical_findings) {
      findingsMap.set(f.canonical_slug, f);
    }
  }

  return report.attack_paths.map((path) => {
    // Map slugs to finding details
    const findingsVM: AttackChainFindingVM[] = path.involved_canonical_slugs.map((slug) => {
      const canonical = findingsMap.get(slug);
      return {
        slug,
        title: canonical?.title || slug.replace(/-/g, " "),
        severity: canonical?.severity || "Medium",
      };
    });

    return {
      id: path.id,
      score: path.attack_path_score,
      narrative: path.narrative,
      exploitability: path.overall_risk_level,
      conditions: path.required_conditions || [],
      context: path.shared_context || [],
      findings: findingsVM,
      verificationStatus: path.active_verification?.status,
      verificationReasoning: path.active_verification?.reasoning,
    };
  });
}

// --- UI COMPONENTS ---

interface AttackChainsPanelProps {
  data: ApiNormalizedAuditReport;
}

export function AttackChainsPanel({ data }: AttackChainsPanelProps) {
  const chains = mapApiToViewModel(data);

  if (chains.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center p-12 text-center border border-border/20 rounded-md bg-[#080808]">
        <ShieldCheck className="h-10 w-10 text-emerald-500 mb-3 opacity-80 shadow-[0_0_15px_rgba(16,185,129,0.3)] rounded-full" />
        <h3 className="text-sm font-bold text-emerald-400 uppercase tracking-widest">No Attack Chains Detected</h3>
        <p className="text-[12px] text-muted-foreground/60 mt-1 max-w-sm">
          The intelligence engine could not find any correlated exploit paths. Your security hygiene prevents multi-stage attacks.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between p-4 bg-[#0a0a0c] border border-border/20 rounded-md shadow-lg">
        <div className="flex items-center gap-3">
          <BrainCircuit className="h-5 w-5 text-purple-400 drop-shadow-[0_0_6px_#c084fc]" />
          <div>
            <h3 className="text-[13px] font-bold uppercase tracking-widest text-foreground">Correlated Attack Chains</h3>
            <p className="text-[11px] text-muted-foreground/60 mt-0.5">Found {chains.length} potential multi-stage exploit paths.</p>
          </div>
        </div>
      </div>

      <div className="flex flex-col gap-6">
        {chains.map((chain) => (
          <AttackChainCard key={chain.id} chain={chain} />
        ))}
      </div>
    </div>
  );
}

function AttackChainCard({ chain }: { chain: AttackChainViewModel }) {
  // Determine styling based on exploitability
  const isActionable = chain.exploitability.toLowerCase() === "actionable";
  const isTheoretical = chain.exploitability.toLowerCase() === "theoretical";
  
  const borderColor = isActionable 
    ? "border-red-500/30" 
    : isTheoretical ? "border-orange-500/30" : "border-yellow-500/30";
    
  const hoverBorderColor = isActionable 
    ? "hover:border-red-500/60" 
    : isTheoretical ? "hover:border-orange-500/60" : "hover:border-yellow-500/60";

  const glowColor = isActionable
    ? "shadow-[0_0_15px_rgba(239,68,68,0.15)]"
    : isTheoretical ? "shadow-[0_0_15px_rgba(249,115,22,0.15)]" : "shadow-[0_0_15px_rgba(234,179,8,0.15)]";

  const badgeColor = isActionable
    ? "bg-red-500/10 text-red-400 border-red-500/20"
    : isTheoretical ? "bg-orange-500/10 text-orange-400 border-orange-500/20" : "bg-yellow-500/10 text-yellow-400 border-yellow-500/20";

  const Icon = isActionable ? ShieldAlert : isTheoretical ? AlertTriangle : ShieldQuestion;

  return (
    <div className={cn("bg-[#080808] border rounded-md overflow-hidden transition-colors", borderColor, hoverBorderColor, glowColor)}>
      {/* Header */}
      <div className="p-4 bg-white/[0.01] flex flex-wrap items-center justify-between gap-4 border-b border-white/[0.05]">
        <div className="flex items-center gap-3">
          <div className={cn("p-1.5 rounded-md border shadow-inner", badgeColor)}>
            <Icon className="h-4 w-4" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <span className="text-[12px] font-bold uppercase tracking-wider text-foreground">Attack Path</span>
              <span className={cn("text-[9px] font-mono px-1.5 py-0.5 rounded border uppercase", badgeColor)}>
                {chain.exploitability}
              </span>
            </div>
            <div className="flex items-center gap-2 mt-1">
              <span className="text-[10px] text-muted-foreground/60 font-mono">Risk Score:</span>
              <span className={cn("text-[11px] font-mono font-bold", isActionable ? "text-red-400" : "text-orange-400")}>{chain.score} / 100</span>
            </div>
          </div>
        </div>
      </div>

      {/* Body */}
      <div className="p-5 flex flex-col lg:flex-row gap-6">
        {/* Narrative & Context */}
        <div className="flex-1 space-y-5">
          <div>
            <h4 className="text-[10px] uppercase tracking-widest text-muted-foreground/50 font-bold mb-2 flex items-center gap-1.5">
              <Info className="h-3 w-3" /> Narrative
            </h4>
            <p className="text-[12px] text-muted-foreground/80 leading-relaxed font-mono bg-[#111] p-3 rounded-md border border-white/[0.02] shadow-inner">
              {chain.narrative}
            </p>
          </div>

          {(chain.conditions.length > 0 || chain.context.length > 0) && (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {chain.conditions.length > 0 && (
                <div>
                  <h4 className="text-[10px] uppercase tracking-widest text-muted-foreground/50 font-bold mb-2 flex items-center gap-1.5">
                    <Target className="h-3 w-3" /> Required Conditions
                  </h4>
                  <ul className="space-y-1.5">
                    {chain.conditions.map((cond, i) => (
                      <li key={i} className="flex items-start gap-2 text-[10px] text-muted-foreground/70 font-mono">
                        <span className="mt-1 h-1 w-1 rounded-full bg-primary/50 shrink-0" />
                        {cond}
                      </li>
                    ))}
                  </ul>
                </div>
              )}
              {chain.context.length > 0 && (
                <div>
                  <h4 className="text-[10px] uppercase tracking-widest text-muted-foreground/50 font-bold mb-2 flex items-center gap-1.5">
                    <Link2 className="h-3 w-3" /> Shared Context
                  </h4>
                  <div className="flex flex-wrap gap-1.5">
                    {chain.context.map((ctx, i) => (
                      <span key={i} className="text-[9px] font-mono bg-white/[0.03] text-muted-foreground/80 border border-white/[0.05] px-2 py-0.5 rounded">
                        {ctx}
                      </span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {chain.verificationStatus && (
            <div className="bg-primary/5 border border-primary/10 rounded-md p-3">
              <h4 className="text-[10px] uppercase tracking-widest text-primary/70 font-bold mb-1 flex items-center gap-1.5">
                <Zap className="h-3 w-3" /> Autonomous Verification: {chain.verificationStatus}
              </h4>
              <p className="text-[10px] text-muted-foreground/70 font-mono">
                {chain.verificationReasoning}
              </p>
            </div>
          )}
        </div>

        {/* Timeline of Findings */}
        <div className="w-full lg:w-72 shrink-0 border-l border-border/10 pl-6 relative">
          <h4 className="text-[10px] uppercase tracking-widest text-muted-foreground/50 font-bold mb-4">Execution Chain</h4>
          <div className="absolute top-10 bottom-4 left-[23px] w-[2px] bg-gradient-to-b from-primary/40 to-transparent" />
          
          <div className="space-y-4">
            {chain.findings.map((finding, idx) => (
              <div key={idx} className="relative z-10 flex gap-3 group">
                <div className="mt-1 flex items-center justify-center h-4 w-4 rounded-full bg-background border-2 border-primary shadow-[0_0_8px_var(--primary)] shrink-0">
                  <div className="h-1 w-1 rounded-full bg-primary" />
                </div>
                <div className="bg-[#111] border border-white/[0.05] rounded-md p-2.5 flex-1 shadow-md group-hover:border-primary/30 transition-colors">
                  <div className="text-[10px] uppercase tracking-wider text-muted-foreground/50 mb-1 font-mono">Step {idx + 1}</div>
                  <div className="text-[11px] font-bold text-foreground/90 capitalize leading-snug">{finding.title}</div>
                  <div className="mt-1.5 flex items-center gap-1.5">
                    <span className="text-[8px] uppercase tracking-wider bg-white/[0.03] text-muted-foreground px-1.5 py-0.5 rounded border border-white/[0.05]">
                      {finding.severity}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
