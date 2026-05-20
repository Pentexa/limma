"use client";

import { useState, Fragment } from "react";
import { cn } from "@/shared/lib/utils";
import { useRuleEngineStatus, useFeedbackStats, useCreateRule, useDeleteRule } from "@/features/manage-rules/model/use-rules";
import {
  Loader2, Shield, BookOpen, BarChart3, CheckCircle, XCircle, Clock, Activity,
  ChevronDown, Info, Zap, Target, AlertTriangle, FileCode, Package, Plus, Trash2
} from "lucide-react";
import { CreateRuleModal } from "./CreateRuleModal";

type Tab = "rules" | "feedback" | "stats";

const TABS: { id: Tab; label: string; icon: React.ElementType }[] = [
  { id: "rules", label: "Active Rules", icon: BookOpen },
  { id: "feedback", label: "Feedback", icon: Activity },
  { id: "stats", label: "Statistics", icon: BarChart3 },
];

/* ── Rule description generator ─────────────────────────────────────────
   Since the API doesn't include a description field, we generate helpful
   context from the rule's name, category, pack, and severity.
   ────────────────────────────────────────────────────────────────────── */

const CATEGORY_DESCRIPTIONS: Record<string, { purpose: string; impact: string; icon: React.ElementType }> = {
  injection: {
    purpose: "Detects injection attacks where user input is processed in an unsafe manner. Covers attack vectors such as SQL, Command, NoSQL, and Template injection.",
    impact: "A successful injection attack can result in database access, command execution, or full server compromise.",
    icon: AlertTriangle,
  },
  xss: {
    purpose: "Detects Cross-Site Scripting (XSS) vulnerabilities. Analyzes Reflected, Stored, and DOM-based XSS variants.",
    impact: "Can lead to session hijacking, credential theft, and malicious content injection.",
    icon: Zap,
  },
  "file-inclusion": {
    purpose: "Detects Local File Inclusion (LFI) and Remote File Inclusion (RFI) vulnerabilities. Aims to prevent unauthorized access to sensitive files on the server.",
    impact: "Carries risks of source code leakage, configuration file exposure, and remote code execution.",
    icon: FileCode,
  },
  ssrf: {
    purpose: "Detects Server-Side Request Forgery (SSRF) vulnerabilities. Prevents the server from sending unauthorized requests to internal networks or external resources.",
    impact: "Creates risks of internal network mapping, metadata endpoint access, and firewall bypass.",
    icon: Target,
  },
  authentication: {
    purpose: "Detects weaknesses in authentication mechanisms. Covers JWT manipulation, weak password policies, and session management flaws.",
    impact: "Creates risks of unauthorized account access, session hijacking, and privilege escalation.",
    icon: Shield,
  },
  "path-traversal": {
    purpose: "Detects path traversal attacks. Prevents unauthorized access to the server file system using '../' sequences.",
    impact: "Carries risks of reading sensitive system files (passwd, config) and potential data leakage.",
    icon: FileCode,
  },
  redirect: {
    purpose: "Detects Open Redirect vulnerabilities. Prevents users from being redirected to malicious sites.",
    impact: "Creates risks of phishing attacks, OAuth token theft, and abuse of user trust.",
    icon: Target,
  },
  graphql: {
    purpose: "Detects GraphQL API security vulnerabilities. Covers introspection leaks, depth attacks, and authorization bypasses.",
    impact: "Carries risks of API schema leakage, denial of service, and data access control bypass.",
    icon: Package,
  },
  xxe: {
    purpose: "Detects XML External Entity (XXE) vulnerabilities. Prevents processing of malicious XML structures.",
    impact: "Creates risks of server file reading, SSRF, and potential remote code execution.",
    icon: FileCode,
  },
};

function getRuleDescription(rule: {
  id: string;
  name: string;
  category: string;
  pack: string;
  default_severity: string;
  default_confidence: string;
  source: string;
}) {
  const categoryKey = rule.category?.toLowerCase() ?? "";
  const nameKey = rule.name?.toLowerCase() ?? "";

  // Try direct category match first
  const match = CATEGORY_DESCRIPTIONS[categoryKey];
  if (match) return match;

  // Try partial match on category or name
  for (const [key, val] of Object.entries(CATEGORY_DESCRIPTIONS)) {
    if (categoryKey.includes(key) || nameKey.includes(key)) return val;
  }

  // Fallback: generate generic description based on severity
  const severityText = rule.default_severity?.toLowerCase();
  const impactLevel = severityText === "critical" || severityText === "high"
    ? "This is a high-priority rule. Vulnerabilities it detects can pose serious security risks."
    : severityText === "medium"
    ? "Detects medium-level security risks. While not directly exploitable, they can be part of chained attacks."
    : "Detects low-level security findings or informational observations.";

  return {
    purpose: `This rule belongs to the "${rule.category}" category and applies the "${rule.name}" security check. It originates from the ${rule.pack} pack.`,
    impact: impactLevel,
    icon: Info,
  };
}

export function RuleEngineScreen() {
  const { data: engineStatus, isLoading: engineLoading } = useRuleEngineStatus();
  const { data: feedbackStats, isLoading: fbLoading } = useFeedbackStats();
  const { mutate: createRule, isPending: isCreating } = useCreateRule();
  const { mutate: deleteRule, isPending: isDeleting } = useDeleteRule();

  const [activeTab, setActiveTab] = useState<Tab>("rules");
  const [expandedRuleId, setExpandedRuleId] = useState<string | null>(null);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);

  const isLoading = engineLoading || fbLoading;

  const handleCreateRule = (payload: { id: string; name: string; yaml_content: string }) => {
    createRule(payload, {
      onSuccess: () => {
        setIsCreateModalOpen(false);
      }
    });
  };

  const handleDeleteRule = (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirm(`Are you sure you want to delete the custom rule '${id}'?`)) {
      deleteRule(id);
    }
  };

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden bg-[#0a0a0c]">
      {/* Top bar */}
      <div className="shrink-0 flex items-center justify-between gap-4 px-5 py-3 border-b border-white/[0.06]">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center h-7 w-7 rounded-lg bg-primary/10 border border-primary/20">
            <Shield className="h-3.5 w-3.5 text-primary" />
          </div>
          <div>
            <h2 className="text-sm font-semibold text-foreground">Rule Engine</h2>
            {engineStatus && (
              <p className="text-[10px] text-muted-foreground/60">
                {engineStatus.total_rules} rules · {engineStatus.active_rules?.length ?? 0} active
              </p>
            )}
          </div>
        </div>
        <div className="flex items-center gap-3">
          {isLoading && (
            <div className="flex items-center gap-1.5 text-[11px] font-medium text-primary animate-pulse mr-2">
              <Loader2 className="h-4 w-4 animate-spin" /> Loading…
            </div>
          )}
          <button
            onClick={() => setIsCreateModalOpen(true)}
            className="flex items-center gap-2 px-3 py-1.5 bg-primary hover:bg-primary/90 text-primary-foreground text-xs font-semibold rounded-md transition-colors"
          >
            <Plus className="w-3.5 h-3.5" />
            New Rule
          </button>
        </div>
      </div>

      {/* Tab bar */}
      <div className="shrink-0 flex items-center gap-2 border-b border-white/[0.06] px-5 py-2.5 overflow-x-auto">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            className={cn(
              "flex items-center gap-2 px-3 py-1.5 text-[12px] font-semibold rounded-lg transition-all duration-200 whitespace-nowrap",
              activeTab === tab.id
                ? "bg-primary/[0.08] text-primary"
                : "bg-transparent text-muted-foreground/60 hover:text-foreground hover:bg-white/[0.02]"
            )}
            onClick={() => setActiveTab(tab.id)}
          >
            <tab.icon className="h-4 w-4 shrink-0 opacity-80" />
            {tab.label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6">
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <div className="flex items-center gap-2 text-primary animate-pulse">
              <Loader2 className="h-5 w-5 animate-spin" />
              <span className="text-[13px] font-medium">Loading rule engine…</span>
            </div>
          </div>
        ) : (
          <div className="max-w-6xl mx-auto">
            {/* Active Rules tab */}
            {activeTab === "rules" && engineStatus?.active_rules && (
              <div className="border border-white/[0.06] rounded-md overflow-hidden bg-black/20">
                <div className="overflow-x-auto">
                  <table className="w-full text-left text-[11px]">
                    <thead className="bg-white/[0.02] border-b border-white/[0.06] text-muted-foreground/60 font-mono">
                      <tr>
                        <th className="px-3 py-2.5 font-medium w-[40px]">Active</th>
                        <th className="px-3 py-2.5 font-medium">Rule</th>
                        <th className="px-3 py-2.5 font-medium w-[100px]">Category</th>
                        <th className="px-3 py-2.5 font-medium w-[80px]">Pack</th>
                        <th className="px-3 py-2.5 font-medium w-[72px] text-center">Severity</th>
                        <th className="px-3 py-2.5 font-medium w-[72px] text-center">Confidence</th>
                        <th className="px-3 py-2.5 font-medium w-[60px]">Version</th>
                        <th className="px-3 py-2.5 font-medium w-[32px]"></th>
                      </tr>
                    </thead>
                    <tbody>
                      {engineStatus.active_rules.map((rule) => {
                        const isExpanded = expandedRuleId === rule.id;
                        const desc = getRuleDescription(rule);
                        const DescIcon = desc.icon;
                        return (
                          <Fragment key={rule.id}>
                            <tr
                              key={rule.id}
                              className={cn(
                                "cursor-pointer transition-colors border-b border-white/[0.04]",
                                isExpanded
                                  ? "bg-primary/[0.04] border-b-transparent"
                                  : "hover:bg-white/[0.02]"
                              )}
                              onClick={() => setExpandedRuleId(isExpanded ? null : rule.id)}
                            >
                              <td className="px-3 py-2.5 text-center">
                                {rule.is_active ? <CheckCircle className="h-3.5 w-3.5 text-emerald-400" /> : <XCircle className="h-3.5 w-3.5 text-muted-foreground/30" />}
                              </td>
                              <td className="px-3 py-2.5">
                                <span className="text-[12px] font-medium text-foreground/90">{rule.name}</span>
                                <span className="text-[9px] font-mono text-muted-foreground/40 block mt-0.5">{rule.id}</span>
                              </td>
                              <td className="px-3 py-2.5 text-[11px] text-muted-foreground/70">{rule.category}</td>
                              <td className="px-3 py-2.5 text-[11px] text-muted-foreground/70 font-mono">{rule.pack}</td>
                              <td className="px-3 py-2.5 text-center">
                                <span className={cn("sev-badge text-[8px]", `sev-badge-${rule.default_severity?.toLowerCase()}`)}>{rule.default_severity}</span>
                              </td>
                              <td className="px-3 py-2.5 text-center text-[11px] text-muted-foreground/70 font-mono tabular-nums">{rule.default_confidence}</td>
                              <td className="px-3 py-2.5 text-[10px] font-mono text-muted-foreground/50">{rule.version}</td>
                              <td className="px-3 py-2.5 text-center">
                                <ChevronDown className={cn(
                                  "h-3.5 w-3.5 text-muted-foreground/40 transition-transform duration-300",
                                  isExpanded && "rotate-180 text-primary"
                                )} />
                              </td>
                            </tr>

                            {/* Expandable Detail Row */}
                            {isExpanded && (
                              <tr key={`${rule.id}-detail`}>
                                <td colSpan={8} className="p-0">
                                  <div className="bg-primary/[0.02] border-b border-white/[0.06] animate-in slide-in-from-top-1 fade-in duration-200">
                                    <div className="px-6 py-5">
                                      <div className="grid grid-cols-[1fr_1fr] gap-6">
                                        {/* Left: Description */}
                                        <div className="space-y-4">
                                          <div className="flex items-start gap-3">
                                            <div className="flex items-center justify-center h-8 w-8 rounded-lg bg-primary/10 border border-primary/20 shrink-0 mt-0.5">
                                              <DescIcon className="h-4 w-4 text-primary" />
                                            </div>
                                            <div className="space-y-2">
                                              <h4 className="text-[12px] font-bold text-foreground/90">What Does It Do?</h4>
                                              <p className="text-[11px] leading-relaxed text-muted-foreground/80">{desc.purpose}</p>
                                            </div>
                                          </div>

                                          <div className="flex items-start gap-3">
                                            <div className="flex items-center justify-center h-8 w-8 rounded-lg bg-red-500/10 border border-red-500/20 shrink-0 mt-0.5">
                                              <AlertTriangle className="h-4 w-4 text-red-400" />
                                            </div>
                                            <div className="space-y-2">
                                              <h4 className="text-[12px] font-bold text-foreground/90">What Is the Impact?</h4>
                                              <p className="text-[11px] leading-relaxed text-muted-foreground/80">{desc.impact}</p>
                                            </div>
                                          </div>
                                        </div>

                                        {/* Right: Technical details */}
                                        <div className="space-y-3">
                                          <div className="flex items-center justify-between mb-3">
                                            <h4 className="text-[10px] font-bold uppercase tracking-widest text-muted-foreground/50">Technical Details</h4>
                                            {(rule.source === "user-custom" || rule.pack === "custom") && (
                                              <button
                                                onClick={(e) => handleDeleteRule(rule.id, e)}
                                                disabled={isDeleting}
                                                className="flex items-center gap-1.5 px-2 py-1 bg-red-500/10 hover:bg-red-500/20 border border-red-500/20 text-red-400 text-[10px] font-bold uppercase tracking-widest rounded transition-colors disabled:opacity-50"
                                              >
                                                <Trash2 className="w-3 h-3" />
                                                Delete
                                              </button>
                                            )}
                                          </div>
                                          <div className="grid grid-cols-2 gap-2">
                                            {[
                                              { label: "Source", value: rule.source || "Built-in" },
                                              { label: "Pack", value: rule.pack },
                                              { label: "Category", value: rule.category },
                                              { label: "Version", value: rule.version },
                                              { label: "Default Severity", value: rule.default_severity },
                                              { label: "Confidence", value: rule.default_confidence },
                                            ].map((item) => (
                                              <div key={item.label} className="bg-black/30 border border-white/[0.04] rounded-md px-3 py-2">
                                                <span className="text-[8px] uppercase tracking-widest text-muted-foreground/40 font-bold block">{item.label}</span>
                                                <span className="text-[11px] font-mono text-foreground/80 mt-0.5 block">{item.value}</span>
                                              </div>
                                            ))}
                                          </div>

                                          {/* Feedback stats for this rule if available */}
                                          {feedbackStats?.rule_stats?.[rule.id] && (
                                            <div className="mt-3 bg-black/30 border border-white/[0.04] rounded-md px-3 py-2.5">
                                              <span className="text-[8px] uppercase tracking-widest text-muted-foreground/40 font-bold block mb-2">Feedback Summary</span>
                                              <div className="flex items-center gap-4 text-[11px] font-mono">
                                                <span className="text-emerald-400">
                                                  {feedbackStats.rule_stats[rule.id].confirmed} <span className="text-[9px] text-muted-foreground/50">confirmed</span>
                                                </span>
                                                <span className="text-red-400">
                                                  {feedbackStats.rule_stats[rule.id].false_positives} <span className="text-[9px] text-muted-foreground/50">FP</span>
                                                </span>
                                                <span className="text-muted-foreground/60">
                                                  {feedbackStats.rule_stats[rule.id].ignored} <span className="text-[9px] text-muted-foreground/50">ignored</span>
                                                </span>
                                                <span className="ml-auto">
                                                  <span className={cn("font-bold",
                                                    feedbackStats.rule_stats[rule.id].reputation_score >= 80 ? "text-emerald-400" :
                                                    feedbackStats.rule_stats[rule.id].reputation_score >= 50 ? "text-foreground/80" : "text-red-400"
                                                  )}>
                                                    {feedbackStats.rule_stats[rule.id].reputation_score.toFixed(0)}%
                                                  </span>
                                                  <span className="text-[9px] text-muted-foreground/50 ml-1">reputation</span>
                                                </span>
                                              </div>
                                            </div>
                                          )}
                                        </div>
                                      </div>
                                    </div>
                                  </div>
                                </td>
                              </tr>
                            )}
                          </Fragment>
                        );
                      })}
                    </tbody>
                  </table>
                </div>
                {engineStatus.active_rules.length === 0 && (
                  <div className="flex flex-col items-center justify-center py-12 text-muted-foreground/30 px-6 text-center">
                    <div className="h-14 w-14 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center mb-3">
                      <BookOpen className="h-6 w-6 opacity-50" />
                    </div>
                    <span className="text-[13px] font-medium text-muted-foreground/50">No active rules</span>
                    <span className="text-[11px] mt-1 text-muted-foreground/30">Rules will appear here once the engine is configured.</span>
                  </div>
                )}
              </div>
            )}

            {/* Feedback tab */}
            {activeTab === "feedback" && feedbackStats?.recent_feedback && (
              <div className="border border-white/[0.06] rounded-md overflow-hidden bg-black/20">
                <div className="flex items-center justify-between px-4 py-3 border-b border-white/[0.06] bg-white/[0.01]">
                  <span className="text-[12px] font-semibold text-foreground/80">Recent Feedback</span>
                </div>
                <div className="divide-y divide-white/[0.04]">
                  {feedbackStats.recent_feedback.length === 0 ? (
                    <div className="flex flex-col items-center justify-center py-12 text-muted-foreground/30 px-6 text-center">
                      <div className="h-14 w-14 rounded-2xl bg-white/[0.03] border border-white/[0.06] flex items-center justify-center mb-3">
                        <Activity className="h-6 w-6 opacity-50" />
                      </div>
                      <span className="text-[13px] font-medium text-muted-foreground/50">No feedback yet</span>
                      <span className="text-[11px] mt-1 text-muted-foreground/30">Feedback entries will appear here as scans are reviewed.</span>
                    </div>
                  ) : feedbackStats.recent_feedback.map((fb, i) => (
                    <div key={i} className="flex items-center gap-3 px-4 py-3 hover:bg-white/[0.02] transition-colors">
                      <span className={cn(
                        "text-[9px] font-mono font-bold uppercase px-2 py-0.5 rounded border shrink-0 w-[90px] text-center",
                        fb.action === "confirmed" ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/20" :
                        fb.action === "false_positive" ? "bg-red-500/10 text-red-400 border-red-500/20" :
                        "bg-white/[0.04] text-muted-foreground/60 border-white/[0.06]"
                      )}>{fb.action}</span>
                      <span className="text-[11px] font-mono text-foreground/70 truncate">{fb.rule_id}</span>
                      <span className="text-[11px] font-mono text-muted-foreground/40 truncate flex-1">{fb.target_url}</span>
                      <span className="text-[10px] text-muted-foreground/40 shrink-0 flex items-center gap-1">
                        <Clock className="h-3 w-3" />{new Date(fb.timestamp).toLocaleDateString()}
                      </span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Stats tab */}
            {activeTab === "stats" && feedbackStats?.rule_stats && (
              <div className="space-y-4">
                {/* Summary metric pills */}
                <div className="flex flex-wrap items-center gap-3">
                  <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-white/[0.06] px-3 py-1.5 rounded shadow-sm">
                    <span className="text-[15px] font-mono font-bold text-foreground">{feedbackStats.total_feedback_entries}</span>
                    <span className="text-[9px] uppercase tracking-widest text-muted-foreground/60 font-bold">Total Entries</span>
                  </div>
                  <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-white/[0.06] px-3 py-1.5 rounded shadow-sm">
                    <span className="text-[15px] font-mono font-bold text-foreground">{Object.keys(feedbackStats.rule_stats).length}</span>
                    <span className="text-[9px] uppercase tracking-widest text-muted-foreground/60 font-bold">Rules Tracked</span>
                  </div>
                </div>

                <div className="border border-white/[0.06] rounded-md overflow-hidden bg-black/20">
                  <div className="flex items-center justify-between px-4 py-3 border-b border-white/[0.06] bg-white/[0.01]">
                    <span className="text-[12px] font-semibold text-foreground/80">Per-Rule Statistics</span>
                  </div>
                  <div className="overflow-x-auto">
                    <table className="w-full text-left text-[11px] whitespace-nowrap">
                      <thead className="bg-white/[0.02] border-b border-white/[0.06] text-muted-foreground/60 font-mono">
                        <tr>
                          <th className="px-3 py-2.5 font-medium">Rule</th>
                          <th className="px-3 py-2.5 font-medium w-[60px] text-center">Total</th>
                          <th className="px-3 py-2.5 font-medium w-[72px] text-center">Confirmed</th>
                          <th className="px-3 py-2.5 font-medium w-[52px] text-center">FP</th>
                          <th className="px-3 py-2.5 font-medium w-[60px] text-center">Ignored</th>
                          <th className="px-3 py-2.5 font-medium w-[80px] text-center">Reputation</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-white/[0.04]">
                        {Object.entries(feedbackStats.rule_stats).map(([ruleId, stats]) => (
                          <tr key={ruleId} className="hover:bg-white/[0.02] transition-colors">
                            <td className="px-3 py-2.5">
                              <span className="text-[12px] font-medium text-foreground/90">{stats.rule_name}</span>
                              <span className="text-[9px] font-mono text-muted-foreground/40 block mt-0.5">{ruleId}</span>
                            </td>
                            <td className="px-3 py-2.5 text-center font-mono text-[11px] tabular-nums text-foreground/80">{stats.total_feedback}</td>
                            <td className="px-3 py-2.5 text-center font-mono text-[11px] tabular-nums text-emerald-400">{stats.confirmed}</td>
                            <td className="px-3 py-2.5 text-center font-mono text-[11px] tabular-nums text-red-400">{stats.false_positives}</td>
                            <td className="px-3 py-2.5 text-center font-mono text-[11px] tabular-nums text-muted-foreground/70">{stats.ignored}</td>
                            <td className="px-3 py-2.5 text-center">
                              <span className={cn("font-mono text-[11px] font-bold tabular-nums",
                                stats.reputation_score >= 80 ? "text-emerald-400" : stats.reputation_score >= 50 ? "text-foreground/80" : "text-red-400"
                              )}>{stats.reputation_score.toFixed(0)}%</span>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              </div>
            )}
          </div>
        )}
      </div>
      <CreateRuleModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        onSubmit={handleCreateRule}
        isSubmitting={isCreating}
      />
    </div>
  );
}
