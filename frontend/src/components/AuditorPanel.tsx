import React, { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { ShieldAlert, Terminal, Bug, Link, ShieldCheck, Globe, Code, Server, CheckCircle2, XCircle, AlertTriangle, Info, ChevronDown, ChevronRight, Fingerprint, TrendingUp, Zap, ArrowUpDown, EyeOff, Eye, ShieldX, ArrowDown, ArrowUp, Minus, Activity, Network, Lock, Search, FileText, Download, Upload, GitBranch, RefreshCw, Layers, Database, Maximize2, X, ArrowRight, Play, Plus, Trash2, Shield, Save, Key, Tag, Crosshair, Target, StopCircle, RefreshCcw, LogOut, Loader2, FastForward, PlayCircle, Cpu, Wifi, MapPin, Hash, ZapOff, Anchor, Inbox, MoreHorizontal, MessageSquare, PlusCircle, PenTool, Image, User, Users, Check, ExternalLink, Settings, TrendingDown, History, Clock, ShieldOff, HardDrive, Compass } from "lucide-react";
import clsx from "clsx";

export default function AuditorPanel({ auditData, scanStrategy }: { auditData: any, scanStrategy?: any[] }) {
  const [activeTab, setActiveTab] = useState<'strategy' | 'priority' | 'attack_paths' | 'correlations' | 'rules' | 'unique' | 'findings' | 'logs'>('strategy');
  const [expandedFindings, setExpandedFindings] = useState<Record<string, boolean>>({});
  const [showSuppressed, setShowSuppressed] = useState(false);
  const [submittingFeedback, setSubmittingFeedback] = useState<string | null>(null);
  const [completedFeedbacks, setCompletedFeedbacks] = useState<Record<string, string>>({});

  const handleFeedback = async (cf: any, action: string) => {
    try {
      setSubmittingFeedback(`${cf.id}-${action}`);

      const formatModule = (m: string) => {
         return m.split('_').map(word => word.charAt(0).toUpperCase() + word.slice(1)).join('');
      };
      let mods = [...(cf.contributing_modules || [])].map(formatModule);
      mods.sort();
      const signature = `${cf.canonical_slug}_[${mods.join(',')}]`;
      
      await fetch('http://localhost:8900/api/feedback', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ signature, action })
      });
      
      setCompletedFeedbacks(prev => ({ ...prev, [cf.id]: action }));
    } catch (e) {
      console.error("Failed to submit feedback", e);
    } finally {
      setSubmittingFeedback(null);
    }
  };

  // Combine and sort priority items
  const prioritizedItems = React.useMemo(() => {
    if (!auditData) return [];
    
    let items: any[] = [];
    
    // Add Attack Paths
    if (auditData.attack_paths) {
      items = items.concat(auditData.attack_paths.filter((ap: any) => ap.priority_assessment).map((ap: any) => ({
        ...ap,
        _type: 'attack_path'
      })));
    }
    
    // Add Canonical Findings
    if (auditData.canonical_findings) {
      items = items.concat(auditData.canonical_findings.filter((cf: any) => cf.priority_assessment).map((cf: any) => ({
        ...cf,
        _type: 'canonical_finding'
      })));
    }

    // Sort descending by score
    return items.sort((a, b) => b.priority_assessment.priority_score - a.priority_assessment.priority_score);
  }, [auditData]);

  const toggleFinding = (id: string) => {
    setExpandedFindings(prev => ({ ...prev, [id]: !prev[id] }));
  };

  const tabs = [
    { id: 'strategy', label: 'Scan Strategy', icon: Compass, count: scanStrategy?.length || 0 },
    { id: 'priority', label: 'Priority Action Items', icon: Crosshair, count: prioritizedItems.length },
    { id: 'attack_paths', label: 'Attack Paths', icon: TrendingUp, count: auditData?.attack_paths?.length || 0 },
    { id: 'correlations', label: 'Correlations', icon: Link, count: auditData?.correlations?.length || 0 },
    { id: 'rules', label: 'Rule Engine', icon: ShieldCheck, count: auditData?.rule_results?.length || 0 },
    { id: 'unique', label: 'Unique Findings', icon: ShieldAlert, count: auditData?.canonical_findings?.length || 0 },
    { id: 'findings', label: 'Raw Findings', icon: Bug, count: auditData?.findings?.length || 0 },
    { id: 'logs', label: 'Process Logs', icon: Terminal, count: auditData?.normalization_log?.length || 0 }
  ];

  if (!auditData) return null;

  return (
    <div className="mt-8 bg-[#0a0a0c] border border-white/10 rounded-2xl overflow-hidden shadow-2xl relative">
      {/* Decorative Glow */}
      <div className="absolute top-0 left-1/2 w-full h-[1px] bg-gradient-to-r from-transparent via-accent-cyan/50 to-transparent -translate-x-1/2" />

      {/* Header Overview - Clean Grid */}
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 border-b border-white/5 divide-y md:divide-y-0 md:divide-x divide-white/5 bg-white/[0.02]">
        <div className="p-5 flex flex-col justify-center col-span-2 md:col-span-1">
          <h2 className="text-lg font-bold text-white tracking-tight flex items-center gap-2">
            <ShieldAlert className="w-5 h-5 text-accent-cyan" />
            Security Auditor
          </h2>
          <p className="text-[10px] text-gray-500 mt-0.5">Context-Aware Engine</p>
        </div>
        
        <div className="p-5 flex flex-col justify-center">
          <p className="text-[9px] uppercase tracking-widest text-gray-500 font-semibold mb-1">Analyzed</p>
          <div className="text-2xl font-light text-white">{auditData.total_findings || 0} <span className="text-xs text-gray-500">signals</span></div>
        </div>
        
        <div className="p-5 flex flex-col justify-center">
          <p className="text-[9px] uppercase tracking-widest text-green-500/70 font-semibold mb-1">False Positive</p>
          <div className="text-2xl font-light text-green-400">{auditData.rejected_findings || 0} <span className="text-xs text-green-500/50">filtered</span></div>
        </div>
        
        <div className="p-5 flex flex-col justify-center">
          <p className="text-[9px] uppercase tracking-widest text-red-500/70 font-semibold mb-1">Verified</p>
          <div className="text-2xl font-light text-red-400">{auditData.accepted_findings || 0} <span className="text-xs text-red-500/50">threats</span></div>
        </div>

        <div className="p-5 flex flex-col justify-center">
          <p className="text-[9px] uppercase tracking-widest text-amber-500/70 font-semibold mb-1">Overall Risk</p>
          <div className="text-2xl font-light text-amber-400">{auditData.scoring_stats?.overall_risk_score?.toFixed(0) || '—'} <span className="text-xs text-amber-500/50">/ 100</span></div>
        </div>

        <div className="p-5 flex flex-col justify-center">
          <p className="text-[9px] uppercase tracking-widest text-purple-500/70 font-semibold mb-1">Context</p>
          <div className="flex items-center gap-2">
            {auditData.context_stats?.elevated > 0 && <span className="text-xs text-green-400 font-mono">↑{auditData.context_stats.elevated}</span>}
            {auditData.context_stats?.downgraded > 0 && <span className="text-xs text-yellow-400 font-mono">↓{auditData.context_stats.downgraded}</span>}
            {auditData.context_stats?.suppressed > 0 && <span className="text-xs text-gray-500 font-mono">×{auditData.context_stats.suppressed}</span>}
          </div>
        </div>
      </div>

      {/* Navigation Tabs */}
      <div className="flex px-4 pt-4 border-b border-white/5 overflow-x-auto custom-scrollbar bg-[#0f0f13]">
        {tabs.map(tab => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id as any)}
            className={clsx(
              "flex items-center gap-2 px-5 py-3 text-sm font-medium transition-all relative whitespace-nowrap",
              activeTab === tab.id 
                ? "text-accent-cyan" 
                : "text-gray-500 hover:text-gray-300"
            )}
          >
            <tab.icon className="w-4 h-4" />
            {tab.label}
            <span className={clsx(
              "px-2 py-0.5 rounded-full text-xs font-mono ml-1",
              activeTab === tab.id ? "bg-accent-cyan/10 text-accent-cyan" : "bg-white/5 text-gray-400"
            )}>
              {tab.count}
            </span>
            {activeTab === tab.id && (
              <motion.div layoutId="auditor-tab" className="absolute bottom-0 left-0 right-0 h-0.5 bg-accent-cyan" />
            )}
          </button>
        ))}
      </div>

      {/* Content Area */}
      <div className="p-0 bg-[#0a0a0c]">
        <AnimatePresence mode="wait">

          {/* SCAN STRATEGY VIEW */}
          {activeTab === 'strategy' && (
            <motion.div key="strategy" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="p-6 bg-gradient-to-b from-[#0a0a0c] to-transparent min-h-screen">
              {!scanStrategy || scanStrategy.length === 0 ? (
                <div className="p-12 text-center text-gray-500 bg-[#0f0f13] rounded-xl border border-white/5">No strategy decisions were recorded during this scan.</div>
              ) : (
                <div className="flex flex-col gap-4">
                  {scanStrategy.map((decision: any, i: number) => {
                    const isDeep = decision.priority === 'deep_analysis';
                    const isStandard = decision.priority === 'standard';
                    
                    return (
                      <div key={`strat-${i}`} className="bg-[#111115] border border-white/5 rounded-xl overflow-hidden shadow-lg flex p-4 items-center gap-6">
                        {/* Scope Indicator */}
                        <div className="flex flex-col items-center justify-center shrink-0 w-20">
                           <span className={clsx("text-[10px] uppercase font-bold tracking-widest", 
                             isDeep ? "text-red-400" : isStandard ? "text-blue-400" : "text-gray-500")}
                           >
                             DEPTH L{decision.adaptive_scan_depth}
                           </span>
                           <div className="w-full flex gap-1 mt-1 justify-center">
                             {[...Array(5)].map((_, idx) => (
                               <div key={idx} className={clsx("h-1.5 w-3 rounded-full",
                                 idx < decision.adaptive_scan_depth ? (isDeep ? "bg-red-500" : isStandard ? "bg-blue-500" : "bg-gray-500") : "bg-white/5"
                               )} />
                             ))}
                           </div>
                        </div>

                        {/* Target Info */}
                        <div className="flex-1 min-w-0 border-l border-white/10 pl-6 space-y-1.5">
                           <h4 className="text-sm font-mono text-gray-200 truncate">{decision.target}</h4>
                           <div className="flex flex-wrap gap-2">
                             {decision.reasoning.map((r: string, ridx: number) => (
                               <span key={ridx} className="bg-white/5 border border-white/10 text-gray-400 px-2 py-0.5 rounded text-[10px] flex items-center gap-1.5">
                                 {r.toLowerCase().includes('deep') ? <Target className="w-3 h-3 text-red-400"/> : r.toLowerCase().includes('deprioritized') ? <FastForward className="w-3 h-3 text-gray-500"/> : <Activity className="w-3 h-3 text-blue-400"/>}
                                 {r}
                               </span>
                             ))}
                           </div>
                        </div>
                      </div>
                    )
                  })}
                </div>
              )}
            </motion.div>
          )}
          
          {/* PRIORITY VIEW */}
          {activeTab === 'priority' && (
            <motion.div key="priority" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="p-6 bg-gradient-to-b from-[#0a0a0c] to-transparent min-h-screen">
              {prioritizedItems.length === 0 ? (
                <div className="p-12 text-center text-gray-500 bg-[#0f0f13] rounded-xl border border-white/5">No prioritized threats found. Run a new scan to generate priority rankings.</div>
              ) : (
                <div className="flex flex-col gap-4">
                  {prioritizedItems.map((item, i) => {
                    const isAttackPath = item._type === 'attack_path';
                    
                    const pLevel = item.priority_assessment.priority_level;
                    const pScore = item.priority_assessment.priority_score;
                    
                    const badgeStyles = 
                      pLevel === 'critical' ? 'bg-red-600/20 text-red-400 border-red-500/50 shadow-[0_0_15px_-3px_rgba(239,68,68,0.3)]' :
                      pLevel === 'high' ? 'bg-orange-600/20 text-orange-400 border-orange-500/50' :
                      pLevel === 'medium' ? 'bg-yellow-600/20 text-yellow-400 border-yellow-500/50' :
                      'bg-gray-600/20 text-gray-400 border-gray-500/50';

                    return (
                      <div key={`prio-${i}`} className="bg-[#111115] border border-white/5 rounded-xl overflow-hidden hover:bg-white/[0.01] transition-colors">
                        <div 
                          className={clsx("p-5 flex flex-col md:flex-row items-start md:items-center gap-4", !isAttackPath && "cursor-pointer")}
                          onClick={() => !isAttackPath && toggleFinding(`prio-${i}`)}
                        >
                          {/* Rank & Score Box */}
                          <div className={clsx("flex flex-col items-center justify-center w-16 h-16 rounded-xl border shrink-0", badgeStyles)}>
                            <span className="text-[10px] uppercase font-bold tracking-widest opacity-80 mb-0.5">{pLevel}</span>
                            <span className="text-xl font-bold font-mono leading-none">{pScore}</span>
                          </div>

                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2 mb-1">
                              {isAttackPath ? (
                                <span className="bg-red-500/10 text-red-400 px-1.5 py-0.5 rounded text-[8px] uppercase tracking-widest border border-red-500/20 flex items-center gap-1">
                                  <TrendingUp className="w-2.5 h-2.5" /> Attack Path
                                </span>
                              ) : (
                                <span className={clsx("px-1.5 py-0.5 rounded text-[8px] uppercase tracking-widest border flex items-center gap-1", 
                                  item.severity === 'critical' || item.severity === 'high' ? "bg-orange-500/10 text-orange-400 border-orange-500/20" : "bg-blue-500/10 text-blue-400 border-blue-500/20"
                                )}>
                                  <Bug className="w-2.5 h-2.5" /> Canonical Finding
                                </span>
                              )}
                            </div>
                            <h3 className="text-sm font-medium text-gray-200 truncate">
                              {isAttackPath ? item.narrative : item.title}
                            </h3>
                            <div className="text-xs text-gray-500 flex items-center gap-3 mt-2 flex-wrap">
                              {isAttackPath && (
                                <span className="flex items-center gap-1"><Layers className="w-3 h-3"/> {item.involved_canonical_slugs?.length || 0} Steps Linked</span>
                              )}
                              {!isAttackPath && (
                                <span className="flex items-center gap-1 text-gray-400"><Code className="w-3 h-3"/> {item.canonical_slug}</span>
                              )}
                              <span className={clsx("flex flex-wrap items-center gap-1.5")}>
                                {item.priority_assessment.reasoning.slice(0, 2).map((reason: string, ridx: number) => (
                                  <span key={ridx} className={clsx("text-[9px] uppercase tracking-wider font-bold border rounded px-1.5 py-0.5", 
                                      reason.toLowerCase().includes('boosted') || reason.toLowerCase().includes('actionable') || reason.toLowerCase().includes('auth') || reason.toLowerCase().includes('sensitive') ? "bg-emerald-500/10 text-emerald-400 border-emerald-500/20" :
                                      reason.toLowerCase().includes('dampened') || reason.toLowerCase().includes('penalized') || reason.toLowerCase().includes('unverified') || reason.toLowerCase().includes('inert') ? "bg-red-500/10 text-red-400 border-red-500/20" :
                                      "bg-gray-500/10 text-gray-400 border-gray-500/20"
                                  )}>
                                    {reason}
                                  </span>
                                ))}
                                {item.priority_assessment.reasoning.length > 2 && <span className="text-[10px] text-gray-600">+{item.priority_assessment.reasoning.length - 2} more signals</span>}
                              </span>
                            </div>
                          </div>
                          
                          {/* Active Verification Status Mirror */}
                          <div className="shrink-0 flex items-center justify-end w-32 border-l border-white/5 pl-4">
                            {item.active_verification ? (
                               item.active_verification.status === 'verified_actionable' ? (
                                 <div className="flex flex-col items-end text-right">
                                   <span className="flex items-center gap-1 text-red-500 text-[10px] font-bold uppercase tracking-widest bg-red-500/10 px-1.5 py-0.5 rounded border border-red-500/20 mb-1"><ShieldAlert className="w-3 h-3"/> Positively Verified</span>
                                   <span className="text-[8px] text-gray-500 uppercase">Live Vulnerability</span>
                                 </div>
                               ) : item.active_verification.status === 'verified_inert' ? (
                                 <div className="flex flex-col items-end text-right">
                                   <span className="flex items-center gap-1 text-green-500 text-[10px] font-bold uppercase tracking-widest bg-green-500/10 px-1.5 py-0.5 rounded border border-green-500/20 mb-1"><ShieldCheck className="w-3 h-3"/> Verified Inert</span>
                                   <span className="text-[8px] text-gray-500 uppercase">Blocked by environment</span>
                                 </div>
                               ) : (
                                 <div className="flex flex-col items-end text-right">
                                   <span className="flex items-center gap-1 text-yellow-500 text-[10px] font-bold uppercase tracking-widest bg-yellow-500/10 px-1.5 py-0.5 rounded border border-yellow-500/20 mb-1"><Activity className="w-3 h-3"/> Inconclusive</span>
                                   <span className="text-[8px] text-gray-500 uppercase">Partial / Contextual verification</span>
                                 </div>
                               )
                            ) : (
                               <div className="text-[10px] text-gray-600 uppercase font-bold tracking-widest flex items-center gap-1"><Clock className="w-3 h-3"/> Unverified</div>
                            )}
                          </div>
                        </div>

                        {/* Learning Impact & Actions Panel (Expanded) */}
                        {!isAttackPath && expandedFindings[`prio-${i}`] && (
                          <div className="bg-black/50 border-t border-white/5 p-4 flex flex-col md:flex-row gap-4 justify-between items-start md:items-center">
                            <div className="flex-1">
                              {item.learning_impact && (
                                <div className="text-[10px] uppercase font-bold tracking-widest text-emerald-400 mb-2 flex items-center gap-1.5 bg-emerald-500/10 px-2 py-1 rounded w-fit border border-emerald-500/20">
                                  <Activity className="w-3 h-3" /> Learning Impact: {item.learning_impact}
                                </div>
                              )}
                              <p className="text-xs text-gray-400">Marking this finding helps the autonomous engine learn its real-world reliability. Changes will affect future priority and confidence weights.</p>
                            </div>
                            <div className="flex flex-wrap items-center gap-2 shrink-0">
                               {completedFeedbacks[item.id] ? (
                                 <div className="flex items-center gap-2 text-emerald-400 bg-emerald-500/10 border border-emerald-500/20 px-3 py-1.5 rounded text-[10px] uppercase font-bold tracking-widest">
                                   <CheckCircle2 className="w-3 h-3" /> Feedback Saved
                                 </div>
                               ) : (
                                 <>
                                   <button onClick={() => handleFeedback(item, 'verified_true_positive')} disabled={submittingFeedback === `${item.id}-verified_true_positive`} className="bg-green-500/10 hover:bg-green-500/20 text-green-400 border border-green-500/20 px-3 py-1.5 rounded transition text-[10px] uppercase tracking-widest font-bold flex items-center gap-1.5 disabled:opacity-50">
                                     <CheckCircle2 className="w-3 h-3" /> Confirm TP
                                   </button>
                                   <button onClick={() => handleFeedback(item, 'false_positive')} disabled={submittingFeedback === `${item.id}-false_positive`} className="bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/20 px-3 py-1.5 rounded transition text-[10px] uppercase tracking-widest font-bold flex items-center gap-1.5 disabled:opacity-50">
                                     <ShieldOff className="w-3 h-3" /> False Positive
                                   </button>
                                   <button onClick={() => handleFeedback(item, 'ignored')} disabled={submittingFeedback === `${item.id}-ignored`} className="bg-gray-500/10 hover:bg-gray-500/20 text-gray-400 border border-gray-500/20 px-3 py-1.5 rounded transition text-[10px] uppercase tracking-widest font-bold flex items-center gap-1.5 disabled:opacity-50">
                                     <Minus className="w-3 h-3" /> Ignore
                                   </button>
                                   <button onClick={() => handleFeedback(item, 'fixed')} disabled={submittingFeedback === `${item.id}-fixed`} className="bg-blue-500/10 hover:bg-blue-500/20 text-blue-400 border border-blue-500/20 px-3 py-1.5 rounded transition text-[10px] uppercase tracking-widest font-bold flex items-center gap-1.5 disabled:opacity-50">
                                     <Check className="w-3 h-3" /> Mark Fixed
                                   </button>
                                 </>
                               )}
                            </div>
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </motion.div>
          )}

          {/* ATTACK PATHS */}
          {activeTab === 'attack_paths' && (
            <motion.div key="attack_paths" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="p-6 grid grid-cols-1 gap-6 bg-[url('/bg-patterns/diagonal-stripes.svg')] bg-repeat bg-[length:20px_20px]">
              {auditData.attack_paths?.length === 0 ? (
                <div className="p-12 text-center text-gray-500 bg-[#0f0f13] rounded-xl border border-white/5">No structural attack paths detected. The environment appears highly segmented.</div>
              ) : (
                auditData.attack_paths?.map((path: any, i: number) => (
                  <div key={i} className="bg-gradient-to-r from-[#1a0f14] to-[#0f0f13] border border-red-500/20 rounded-xl overflow-hidden shadow-[0_0_30px_-5px_rgba(239,68,68,0.1)] relative">
                    <div className="absolute top-0 left-0 w-1 h-full bg-red-500/50" />
                    <div className="p-6 border-b border-white/5 flex flex-col md:flex-row gap-4 items-start md:items-center justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-3 mb-2">
                          <span className="flex items-center gap-1.5 px-2 py-0.5 rounded text-[10px] font-bold uppercase tracking-widest bg-red-500/20 text-red-400 border border-red-500/30">
                            <Zap className="w-3 h-3" /> Attack Path Detected
                          </span>
                          <span className="text-xs font-mono text-gray-500">Score: <strong className="text-gray-300">{path.attack_path_score}</strong></span>
                          <span className="text-xs font-mono text-gray-500">Level: <strong className={clsx(
                            path.overall_risk_level === 'actionable' ? "text-red-400" :
                            path.overall_risk_level === 'theoretical' ? "text-yellow-400" : "text-green-400"
                          )}>{path.overall_risk_level}</strong></span>
                        </div>
                        <h3 className="text-lg text-gray-100 leading-tight font-medium">{path.narrative}</h3>
                      </div>
                    </div>
                    <div className="p-6 grid grid-cols-1 md:grid-cols-2 gap-6">
                      <div>
                        <p className="text-[10px] uppercase tracking-widest text-gray-500 font-semibold mb-3 flex items-center gap-2">
                          <TrendingUp className="w-3 h-3" /> Chain Linkage
                        </p>
                        <div className="space-y-3 relative before:absolute before:inset-0 before:ml-2 before:-translate-x-px md:before:mx-auto md:before:translate-x-0 before:h-full before:w-0.5 before:bg-gradient-to-b before:from-red-500/50 before:to-transparent">
                          {path.involved_canonical_slugs?.map((slug: string, j: number) => (
                            <div key={j} className="relative flex items-center justify-between md:justify-normal md:odd:flex-row-reverse group is-active">
                              <div className="flex items-center justify-center w-5 h-5 rounded-full border-2 border-red-500 bg-[#0f0f13] text-red-500 shadow shrink-0 md:order-1 md:group-odd:-translate-x-1/2 md:group-even:translate-x-1/2">
                                <Bug className="w-2.5 h-2.5" />
                              </div>
                              <div className="w-[calc(100%-2.5rem)] md:w-[calc(50%-1.5rem)] p-3 rounded shadow bg-white/5 border border-white/10 group-hover:bg-white/10 transition-colors">
                                <span className="text-xs font-mono text-gray-300">{slug}</span>
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                      
                      <div className="bg-black/40 rounded p-4 border border-white/5 space-y-4">
                        {path.shared_context?.length > 0 && (
                          <div>
                            <p className="text-[10px] uppercase tracking-widest text-gray-500 font-semibold mb-2">Pivotal Context</p>
                            <div className="flex flex-wrap gap-2">
                              {path.shared_context.map((ctx: string, j: number) => (
                                <span key={j} className="text-[10px] font-mono px-2 py-1 bg-white/5 border border-white/10 text-gray-300 rounded">{ctx}</span>
                              ))}
                            </div>
                          </div>
                        )}
                        {path.required_conditions?.length > 0 && (
                          <div>
                            <p className="text-[10px] uppercase tracking-widest text-gray-500 font-semibold mb-2">Required Attack Conditions</p>
                            <ul className="space-y-1">
                              {path.required_conditions.map((req: string, j: number) => (
                                <li key={j} className="text-xs text-gray-400 flex items-start gap-2">
                                  <AlertTriangle className="w-3 h-3 text-yellow-500/70 mt-0.5 shrink-0" /> {req}
                                </li>
                              ))}
                            </ul>
                          </div>
                        )}
                      </div>
                    </div>
                    {path.active_verification && (
                      <div className="bg-blue-900/10 border-t border-blue-500/20 p-6">
                         <div className="flex items-center justify-between mb-4">
                           <p className="text-[10px] text-blue-400 uppercase tracking-widest font-bold flex items-center gap-2"><Eye className="w-3 h-3" /> Autonomous Verification</p>
                           <span className="text-xs font-mono text-blue-300 bg-blue-500/20 px-2 py-1 rounded">Reproducibility: <strong>{path.active_verification.reproducibility_score}%</strong></span>
                         </div>
                         <p className="text-sm text-blue-200 leading-relaxed mb-4">{path.active_verification.reasoning}</p>
                         {path.active_verification.traces?.length > 0 && (
                            <div className="space-y-3">
                              <p className="text-[10px] text-gray-500 uppercase tracking-widest font-bold mb-1 border-t border-blue-500/10 pt-4">Runtime Trace Verification</p>
                              {path.active_verification.traces.map((trace: any, tidx: number) => (
                                <div key={tidx} className="bg-[#0f0f13] border border-blue-500/10 rounded-lg p-3 text-[10px] font-mono text-gray-400">
                                  <div className="flex gap-3 mb-2 items-center">
                                    <span className={clsx("px-2 py-1 rounded text-white font-bold text-[9px]", trace.is_successful ? "bg-red-500/30" : "bg-green-500/30")}>{trace.method}</span>
                                    <span className="text-gray-300 break-all">{trace.endpoint}</span>
                                  </div>
                                  <div className="grid grid-cols-2 gap-4 mt-3 pt-3 border-t border-white/5">
                                    <div className="overflow-x-auto"><span className="text-[8px] text-gray-500 block mb-1">REQUEST</span><pre className="text-gray-400">{trace.request_snapshot}</pre></div>
                                    <div className="overflow-x-auto"><span className="text-[8px] text-gray-500 block mb-1">RESPONSE</span><pre className="text-gray-400">{trace.response_snapshot}</pre></div>
                                  </div>
                                </div>
                              ))}
                            </div>
                         )}
                      </div>
                    )}
                  </div>
                ))
              )}
            </motion.div>
          )}

          {/* CORRELATIONS */}
          {activeTab === 'correlations' && (
            <motion.div key="correlations" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="divide-y divide-white/5">
              {auditData.correlations?.length === 0 ? (
                <div className="p-12 text-center text-gray-500">No complex correlations discovered in this domain.</div>
              ) : (
                auditData.correlations?.map((corr: any, i: number) => (
                  <div key={i} className="p-6 flex flex-col lg:flex-row gap-6 hover:bg-white/[0.01] transition-colors">
                    <div className="lg:w-1/3 flex flex-col gap-3">
                      <div className="flex gap-2 items-center">
                        <span className="px-2 py-1 bg-red-500/10 text-red-400 border border-red-500/20 text-[10px] rounded font-bold uppercase tracking-widest">{corr.correlation_type}</span>
                        <span className="px-2 py-1 bg-white/5 text-gray-400 border border-white/5 text-[10px] rounded uppercase tracking-widest font-mono">Conf: {corr.confidence}</span>
                      </div>
                      <h3 className="text-lg text-white font-medium leading-snug">{corr.summary}</h3>
                      <p className="text-sm text-gray-400 leading-relaxed">{corr.reason?.explanation}</p>
                      <span className="text-[10px] font-mono text-gray-600 mt-2">Code: {corr.reason?.code}</span>
                    </div>

                    <div className="lg:w-2/3 bg-[#0f0f13] rounded-xl border border-white/5 p-5">
                      <p className="text-[10px] uppercase text-gray-500 font-semibold mb-4 tracking-widest flex items-center gap-2">
                        <Link className="w-3 h-3 text-accent-cyan"/> Linked Traces
                      </p>
                      <div className="space-y-3">
                        {corr.linked_findings?.map((link: any, idx: number) => (
                          <div key={idx} className="flex items-start gap-3 group">
                            <div className="w-1.5 h-1.5 rounded-full bg-accent-cyan mt-1.5 opacity-50 relative">
                              {idx !== corr.linked_findings.length -1 && <div className="absolute top-3 left-1/2 w-[1px] h-8 bg-white/5 -translate-x-1/2" />}
                            </div>
                            <div className="text-xs font-mono text-gray-400 group-hover:text-gray-200 transition-colors bg-white/[0.02] px-3 py-2 rounded-lg border border-white/[0.02] flex-1">
                              {link.relationship_note}
                            </div>
                          </div>
                        ))}
                      </div>
                    </div>
                  </div>
                ))
              )}
            </motion.div>
          )}

          {/* RULES */}
          {activeTab === 'rules' && (
            <motion.div key="rules" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="p-6 grid grid-cols-1 xl:grid-cols-2 gap-6">
               {auditData.rule_results?.length === 0 ? (
                <div className="col-span-full p-12 text-center text-gray-500">No rule matches registered.</div>
               ) : (
                  auditData.rule_results?.map((res: any, i: number) => (
                    <div key={i} className="bg-[#0f0f13] border border-white/5 rounded-xl overflow-hidden flex flex-col group">
                      <div className="p-5 border-b border-white/5 flex justify-between items-start">
                        <div>
                          <h3 className="text-white font-medium mb-1">{res.rule_title}</h3>
                          <p className="text-xs text-gray-400 leading-relaxed">{res.summary}</p>
                        </div>
                        <span className={clsx(
                          "px-2 py-1 text-[10px] rounded font-bold uppercase tracking-widest border shrink-0 ml-4",
                          res.outcome === "matched" ? "bg-red-500/10 text-red-400 border-red-500/20" : "bg-yellow-500/10 text-yellow-500 border-yellow-500/20"
                        )}>
                          {res.outcome.replace('_', ' ')}
                        </span>
                      </div>
                      
                      {res.evaluations?.length > 0 && (
                        <div className="p-5 bg-white/[0.01]">
                          <p className="text-[10px] uppercase tracking-widest text-gray-500 font-semibold mb-3">Conditions Breakdown</p>
                          <div className="space-y-2">
                            {res.evaluations.map((ev: any, j: number) => (
                              <div key={j} className="flex items-start gap-3">
                                <div className="mt-0.5">
                                  {ev.is_met ? <CheckCircle2 className="w-4 h-4 text-green-500/70" /> : <XCircle className="w-4 h-4 text-red-500/70" />}
                                </div>
                                <div className="flex-1">
                                  <p className={clsx("text-xs font-medium", ev.is_met ? "text-gray-300" : "text-gray-500")}>{ev.detail}</p>
                                  <div className="flex gap-2 text-[10px] font-mono mt-1 text-gray-600">
                                    <span>[{ev.condition?.condition_type}]</span>
                                    <span>{ev.condition?.expected_value}</span>
                                    <span className="italic opacity-50">{ev.condition?.is_mandatory ? 'Required' : 'Optional'}</span>
                                  </div>
                                </div>
                              </div>
                            ))}
                          </div>
                        </div>
                      )}
                    </div>
                  ))
               )}
            </motion.div>
          )}

          {/* UNIQUE FINDINGS */}
          {activeTab === 'unique' && (
            <motion.div key="unique" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="flex flex-col">
              {auditData.canonical_findings?.length === 0 ? (
                <div className="p-12 text-center text-gray-500">No unique findings computed.</div>
              ) : (
                <div className="divide-y divide-white/5">
                  {auditData.canonical_findings?.map((cf: any, i: number) => {
                    const isExpanded = !!expandedFindings[`canon-${i}`];
                    const sevColor = cf.severity === 'critical' ? 'text-red-400' : 
                                     cf.severity === 'high' ? 'text-orange-400' : 
                                     cf.severity === 'medium' ? 'text-yellow-400' : 
                                     cf.severity === 'low' ? 'text-blue-400' : 'text-gray-400';
                                     
                    const sevBg = cf.severity === 'critical' ? 'bg-red-500/20 text-red-400 border border-red-500/20' :
                                   cf.severity === 'high' ? 'bg-orange-500/15 text-orange-400 border border-orange-500/20' :
                                   cf.severity === 'medium' ? 'bg-yellow-500/15 text-yellow-500 border border-yellow-500/20' :
                                   cf.severity === 'low' ? 'bg-blue-500/15 text-blue-400 border border-blue-500/20' : 'bg-gray-500/15 text-gray-400 border border-gray-500/20';

                    return (
                      <div key={i} className="flex flex-col transition-colors hover:bg-white/[0.01]">
                        <div 
                          className="p-4 px-6 flex items-center justify-between cursor-pointer group"
                          onClick={() => toggleFinding(`canon-${i}`)}
                        >
                          <div className="flex items-center gap-4 flex-1 overflow-hidden">
                            {/* Severity Badge */}
                            <div className="w-16 shrink-0 flex flex-col items-center">
                              <ShieldAlert className={clsx("w-6 h-6 mb-1", sevColor)} />
                              <span className={clsx("text-[8px] uppercase font-bold tracking-widest px-1.5 py-0.5 rounded", sevBg)}>
                                {cf.severity}
                              </span>
                            </div>
                            
                            {/* Exploitability Level */}
                            <div className="w-24 shrink-0 flex flex-col items-center border-l border-white/5 px-2">
                              {cf.exploitability_level === 'actionable' ? (
                                <span className="text-xs font-bold uppercase tracking-wider text-red-500 flex items-center gap-1"><Zap className="w-3 h-3"/> Actionable</span>
                              ) : cf.exploitability_level === 'inert' ? (
                                <span className="text-xs font-bold uppercase tracking-wider text-green-500 flex items-center gap-1"><ShieldCheck className="w-3 h-3"/> Inert</span>
                              ) : (
                                <span className="text-xs font-bold uppercase tracking-wider text-yellow-500">Theoretical</span>
                              )}
                              <span className="text-[7px] uppercase font-bold tracking-widest text-gray-600 mt-1 flex items-center gap-1">
                                {cf.exploitability_score ? `Score: ${cf.exploitability_score}` : 'Exploitability'}
                              </span>
                            </div>
                            
                            {/* Confidence & Merged Status */}
                            <div className="w-20 shrink-0 flex flex-col items-center border-l border-white/5 pl-4 mr-2">
                              <div className="flex items-center gap-1">
                                {cf.confidence_calibration && cf.confidence_calibration.calibration_impact.includes('Reduced') && (
                                   <div title={cf.confidence_calibration.calibration_impact}><TrendingDown className="w-3 h-3 text-red-500 shrink-0" /></div>
                                )}
                                {cf.confidence_calibration && cf.confidence_calibration.calibration_impact.includes('Boosted') && (
                                   <div title={cf.confidence_calibration.calibration_impact}><TrendingUp className="w-3 h-3 text-green-500 shrink-0" /></div>
                                )}
                                <span className={clsx(
                                  "text-xs font-bold uppercase tracking-wider",
                                  cf.confidence === 'certain' ? 'text-green-400' :
                                  cf.confidence === 'firm' ? 'text-blue-400' :
                                  cf.confidence === 'tentative' ? 'text-yellow-400' : 'text-gray-500'
                                )}>{cf.confidence || 'Unk'}</span>
                              </div>
                              <span className="text-[7px] uppercase font-bold tracking-widest text-gray-600 mt-1 whitespace-nowrap">
                                {cf.underlying_findings?.length || 1} Merged
                              </span>
                            </div>

                            <div className="flex flex-col flex-1 min-w-0">
                              <span className="text-sm font-medium text-gray-200 truncate group-hover:text-white transition-colors">{cf.title}</span>
                              <span className="text-xs text-gray-500 font-mono flex items-center gap-2 mt-1 flex-wrap">
                                <span className="bg-white/5 border border-white/5 rounded px-1.5 py-0.5 text-[9px] text-gray-400 uppercase tracking-widest">
                                  {cf.risk_family?.replace('_', ' ')}
                                </span>
                                
                                {cf.contributing_modules?.map((m: string, midx: number) => (
                                  <span key={midx} className="flex items-center gap-1 text-gray-400 text-[10px]">
                                    {m === 'web_scanner' ? <Globe className="w-2.5 h-2.5"/> : m === 'api_discoverer' ? <Code className="w-2.5 h-2.5"/> : <Server className="w-2.5 h-2.5"/>}
                                    {m.replace('_', ' ')}
                                  </span>
                                ))}
                                
                                {cf.affected_routes?.length > 0 && (
                                  <span className="text-accent-cyan/80 text-[10px]">
                                    {cf.affected_routes.join(', ')}
                                  </span>
                                )}
                              </span>
                            </div>
                          </div>
                          
                          <div className="shrink-0 ml-4 flex items-center gap-4">
                            <div className="text-[10px] uppercase tracking-widest text-gray-500 font-mono flex flex-col items-end mr-4">
                              <span>Evidence: <strong className="text-gray-300">{cf.merged_evidence_count}</strong></span>
                              <span>State: <strong className={clsx(
                                cf.verification_status === 'verified_actionable' ? "text-red-400" :
                                cf.verification_status === 'verified_inert' ? "text-green-400" : "text-gray-300"
                              )}>{(cf.verification_status || 'unverified').replace('_', ' ')}</strong></span>
                            </div>
                            {isExpanded ? <ChevronDown className="w-5 h-5 text-gray-500" /> : <ChevronRight className="w-5 h-5 text-gray-600" />}
                          </div>
                        </div>

                        {/* Expanded Underlying Findings */}
                        <AnimatePresence>
                          {isExpanded && (
                            <motion.div 
                              initial={{ height: 0, opacity: 0 }} 
                              animate={{ height: 'auto', opacity: 1 }} 
                              exit={{ height: 0, opacity: 0 }}
                              className="overflow-hidden bg-black border-t border-white/[0.02]"
                            >
                              <div className="p-6">
                                <p className="text-[10px] uppercase text-gray-500 tracking-widest font-bold mb-3 flex items-center gap-1.5">
                                  <Link className="w-3 h-3"/> Raw Detections in Canonical Subgroup: <span className="text-gray-300 ml-1">{cf.canonical_slug}</span>
                                </p>
                                {cf.confidence_calibration && (
                                  <div className="bg-[#0a0a0c] border border-gray-700/50 rounded p-4 mb-4">
                                     <div className="flex items-center justify-between mb-2">
                                       <p className="text-[10px] text-gray-400 uppercase tracking-widest font-bold flex items-center gap-1.5"><History className="w-3 h-3"/> Confidence Calibration Engine</p>
                                       <span className={clsx("text-[10px] uppercase font-bold tracking-widest px-2 py-0.5 rounded",
                                         cf.confidence_calibration.calibration_impact.includes('Reduced') ? "bg-red-500/10 text-red-400" :
                                         cf.confidence_calibration.calibration_impact.includes('Boosted') ? "bg-green-500/10 text-green-400" : "bg-gray-500/10 text-gray-400"
                                       )}>{cf.confidence_calibration.calibration_impact}</span>
                                     </div>
                                     <p className="text-xs text-gray-300 leading-relaxed font-mono">{cf.confidence_calibration.reasoning}</p>
                                     <div className="flex gap-4 mt-3 pt-3 border-t border-white/5">
                                        <div className="text-[9px] uppercase tracking-widest text-gray-500 font-bold">Reliability Coef: <span className="text-gray-300">{cf.confidence_calibration.reliability_coefficient.toFixed(2)}</span></div>
                                        <div className="text-[9px] uppercase tracking-widest text-gray-500 font-bold">Base Conf: <span className="text-gray-300">{cf.confidence_calibration.original_confidence}</span></div>
                                        <div className="text-[9px] uppercase tracking-widest text-gray-500 font-bold">Adjusted Conf: <span className="text-gray-300">{cf.confidence_calibration.adjusted_confidence}</span></div>
                                     </div>
                                  </div>
                                )}
                                {cf.exploitability_reasoning && (
                                  <div className="bg-purple-900/10 border border-purple-500/20 rounded p-4 mb-4">
                                     <p className="text-[10px] text-purple-400 uppercase tracking-widest font-bold mb-1">Runtime Exploitability Assessment</p>
                                     <p className="text-xs text-purple-200 leading-relaxed">{cf.exploitability_reasoning}</p>
                                     {cf.attack_surface_tags?.length > 0 && (
                                        <div className="flex flex-wrap gap-2 mt-3">
                                          {cf.attack_surface_tags.map((tag: string, tidx: number) => (
                                            <span key={tidx} className="bg-purple-500/10 text-purple-300 border border-purple-500/20 px-1.5 py-0.5 rounded text-[9px] uppercase tracking-wider">{tag.replace(/_/g, ' ')}</span>
                                          ))}
                                        </div>
                                     )}
                                  </div>
                                )}
                                {cf.active_verification && (
                                  <div className="bg-blue-900/10 border border-blue-500/20 rounded p-4 mb-4">
                                     <div className="flex items-center justify-between mb-2">
                                       <p className="text-[10px] text-blue-400 uppercase tracking-widest font-bold flex items-center gap-1.5"><Eye className="w-3 h-3" /> Autonomous Verification</p>
                                       <span className="text-[10px] font-mono text-blue-300 bg-blue-500/20 px-2 py-0.5 rounded">Reproducibility: <strong>{cf.active_verification.reproducibility_score}%</strong></span>
                                     </div>
                                     <p className="text-xs text-blue-200 leading-relaxed font-medium mb-3">{cf.active_verification.reasoning}</p>
                                     {cf.active_verification.traces?.length > 0 && (
                                        <div className="space-y-2 mt-2">
                                          <p className="text-[9px] text-gray-500 uppercase tracking-widest font-bold mb-1 border-t border-blue-500/10 pt-3">Raw Trace Snapshots</p>
                                          {cf.active_verification.traces.map((trace: any, tidx: number) => (
                                            <div key={tidx} className="bg-black/40 border border-blue-500/10 rounded p-2 text-[10px] font-mono text-gray-400">
                                              <div className="flex gap-2 mb-1 items-center">
                                                <span className={clsx("px-1.5 py-0.5 rounded text-white font-bold text-[8px]", trace.is_successful ? "bg-red-500/30" : "bg-green-500/30")}>{trace.method}</span>
                                                <span className="text-gray-300 break-all">{trace.endpoint}</span>
                                              </div>
                                              <div className="grid grid-cols-2 gap-2 mt-2 pt-2 border-t border-white/5">
                                                <div className="overflow-x-auto"><span className="text-[8px] text-gray-500 block mb-1">REQ</span><pre className="text-blue-100/50">{trace.request_snapshot}</pre></div>
                                                <div className="overflow-x-auto"><span className="text-[8px] text-gray-500 block mb-1">RES</span><pre className="text-blue-100/50">{trace.response_snapshot}</pre></div>
                                              </div>
                                            </div>
                                          ))}
                                        </div>
                                     )}
                                  </div>
                                )}
                                <div className="space-y-4">
                                  {cf.underlying_findings?.map((uf: any, j: number) => (
                                    <div key={j} className="bg-sidebar-bg/50 border border-sidebar-border rounded-lg p-4 flex flex-col gap-2 relative">
                                      <div className="absolute top-0 left-0 bottom-0 w-1 bg-white/[0.02] rounded-l-lg overflow-hidden">
                                        <div className="w-full h-full bg-accent-cyan/20"></div>
                                      </div>
                                      <div className="flex items-center justify-between ml-2">
                                        <div className="flex items-center gap-2">
                                          <span className="text-xs font-mono px-1.5 py-0.5 rounded bg-white/5 border border-white/5 text-gray-300">
                                            {uf.source_module}
                                          </span>
                                          {uf.affected_path_or_endpoint && (
                                            <span className="text-[10px] font-mono text-gray-500">
                                              {uf.affected_path_or_endpoint}
                                            </span>
                                          )}
                                        </div>
                                        <div className="flex items-center gap-2 text-[10px] uppercase tracking-widest font-bold">
                                          <span className={clsx("px-1.5 py-0.5 rounded border text-black", 
                                            uf.severity === 'critical' ? 'bg-red-400 border-red-400' :
                                            uf.severity === 'high' ? 'bg-orange-400 border-orange-400' :
                                            uf.severity === 'medium' ? 'bg-yellow-400 border-yellow-400' :
                                            uf.severity === 'low' ? 'bg-blue-400 border-blue-400' : 'bg-gray-400 border-gray-400'
                                          )}>{uf.severity}</span>
                                          <span className="text-gray-500 border border-gray-500/20 px-1.5 py-0.5 rounded">{uf.confidence}</span>
                                        </div>
                                      </div>
                                      <span className="text-sm font-medium text-gray-200 ml-2 mt-1">{uf.summary}</span>
                                      {uf.evidence?.length > 0 && (
                                        <div className="ml-2 mt-2 space-y-2">
                                          {uf.evidence.map((ev: any, k: number) => (
                                            <div key={k} className="bg-black/60 rounded p-2 border border-white/5">
                                              <p className="text-[9px] text-gray-500 uppercase font-mono mb-1">{ev.description}</p>
                                              <pre className="text-[10px] text-green-400 font-mono whitespace-pre-wrap">{ev.raw_data}</pre>
                                            </div>
                                          ))}
                                        </div>
                                      )}
                                    </div>
                                  ))}
                                </div>
                              </div>
                            </motion.div>
                          )}
                        </AnimatePresence>
                      </div>
                    );
                  })}
                </div>
              )}
            </motion.div>
          )}

          {/* FINDINGS */}
          {activeTab === 'findings' && (
            <motion.div key="findings" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="flex flex-col">
                {auditData.findings?.length === 0 ? (
                  <div className="p-12 text-center text-gray-500">No raw findings collected.</div>
                ) : (
                  <>
                  {/* Suppressed toggle */}
                  {auditData.context_stats?.suppressed > 0 && (
                    <div className="px-6 py-3 bg-white/[0.02] border-b border-white/5 flex items-center justify-between">
                      <span className="text-[10px] uppercase tracking-widest text-gray-500 font-semibold flex items-center gap-1.5">
                        <EyeOff className="w-3 h-3"/>
                        {auditData.context_stats.suppressed} finding{auditData.context_stats.suppressed > 1 ? 's' : ''} suppressed by context engine
                      </span>
                      <button
                        onClick={() => setShowSuppressed(!showSuppressed)}
                        className="text-[10px] uppercase tracking-widest font-semibold px-3 py-1 rounded border border-white/10 hover:border-white/20 transition-colors flex items-center gap-1.5 text-gray-400 hover:text-gray-300"
                      >
                        {showSuppressed ? <EyeOff className="w-3 h-3"/> : <Eye className="w-3 h-3"/>}
                        {showSuppressed ? 'Hide Suppressed' : 'Show Suppressed'}
                      </button>
                    </div>
                  )}
                  <div className="divide-y divide-white/5">
                    {auditData.findings?.filter((f: any) => {
                      if (!showSuppressed && f.context_assessment?.adjustment === 'suppressed') return false;
                      return true;
                    }).map((f: any, i: number) => {
                      const sevColor = f.severity === 'critical' ? 'text-red-400' : 
                                       f.severity === 'high' ? 'text-orange-400' : 
                                       f.severity === 'medium' ? 'text-yellow-400' : 
                                       f.severity === 'low' ? 'text-blue-400' : 'text-gray-400';
                      
                      const isExpanded = !!expandedFindings[i.toString()];
                      const isSuppressed = f.context_assessment?.adjustment === 'suppressed';
                      const isDowngraded = f.context_assessment?.adjustment === 'downgraded';
                      const isElevated = f.context_assessment?.adjustment === 'elevated';
                      
                      const isHygieneIssue = f.risk_score?.priority_statement?.includes('Hygiene Issue') || f.correlation_is_hygiene_gap;
                      const isLowExploitability = f.risk_score?.priority_statement?.includes('Low-Exploitability Weakness');
                      const isWeakContext = isHygieneIssue || isLowExploitability;

                      return (
                        <div key={i} className={clsx(
                          "flex flex-col transition-colors",
                          isSuppressed ? 'opacity-40 hover:opacity-60 bg-gray-900/30' :
                          isElevated && f.context_assessment?.adjusted_level === 'critical' ? 'bg-red-500/[0.03] hover:bg-red-500/[0.05]' :
                          isElevated ? 'bg-green-500/[0.02] hover:bg-green-500/[0.03]' :
                          isDowngraded || isWeakContext ? 'hover:bg-white/[0.01]' :
                          'hover:bg-white/[0.01]'
                        )}>
                          {/* Row Header */}
                          <div 
                            className="p-4 px-6 flex items-center justify-between cursor-pointer group"
                            onClick={() => toggleFinding(i.toString())}
                          >
                            <div className="flex items-center gap-4 flex-1 overflow-hidden">
                              {/* Severity Badge */}
                              <div className="w-14 shrink-0 flex flex-col items-center">
                                {(() => {
                                  const displayScore = f.context_assessment?.adjusted_score ?? f.risk_score?.total_score ?? 0;
                                  const displayLevel = f.context_assessment?.adjusted_level ?? f.risk_score?.level ?? 'info';
                                  
                                  const levelColor = isWeakContext ? 'text-gray-400' :
                                    displayLevel === 'critical' ? 'text-red-400' :
                                    displayLevel === 'high' ? 'text-orange-400' :
                                    displayLevel === 'medium' ? 'text-yellow-400' :
                                    displayLevel === 'low' ? 'text-blue-400' : 'text-gray-400';
                                    
                                  const levelBg = isWeakContext ? 'bg-gray-500/10 text-gray-400 border border-gray-500/20' :
                                    displayLevel === 'critical' ? 'bg-red-500/20 text-red-400 border border-red-500/20' :
                                    displayLevel === 'high' ? 'bg-orange-500/15 text-orange-400 border border-orange-500/20' :
                                    displayLevel === 'medium' ? 'bg-yellow-500/15 text-yellow-500 border border-yellow-500/20' :
                                    displayLevel === 'low' ? 'bg-blue-500/15 text-blue-400 border border-blue-500/20' : 'bg-gray-500/15 text-gray-400 border border-gray-500/20';
                                    
                                  const overrideLevel = isHygieneIssue ? 'Hygiene Gap' : isLowExploitability ? 'Low Impact' : displayLevel;
                                    
                                  return (
                                    <>
                                      <span className={clsx("text-lg font-bold tabular-nums", levelColor)}>{displayScore}</span>
                                      <span className={clsx("text-[8px] uppercase font-bold tracking-widest px-1.5 py-0.5 rounded mt-0.5", levelBg)}>{overrideLevel}</span>
                                    </>
                                  );
                                })()}
                              </div>
                              
                              {/* Confidence Badge */}
                              <div className="w-16 shrink-0 flex flex-col items-center border-l border-white/5 pl-4 mr-2">
                                <span className={clsx(
                                  "text-xs font-bold uppercase tracking-wider",
                                  f.confidence === 'certain' ? 'text-green-400' :
                                  f.confidence === 'firm' ? 'text-blue-400' :
                                  f.confidence === 'tentative' ? 'text-yellow-400' : 'text-gray-500'
                                )}>{f.confidence || 'Unk'}</span>
                                <span className="text-[7px] uppercase font-bold tracking-widest text-gray-600 mt-1">Confidence</span>
                              </div>

                              <div className="flex flex-col flex-1 min-w-0">
                                <span className="text-sm font-medium text-gray-200 truncate group-hover:text-white transition-colors">{f.summary}</span>
                                <span className="text-xs text-gray-500 font-mono flex items-center gap-2 mt-0.5 flex-wrap">
                                  {f.source_module === 'web_scanner' ? <Globe className="w-3 h-3"/> : f.source_module === 'api_discoverer' ? <Code className="w-3 h-3"/> : <Server className="w-3 h-3"/>}
                                  {f.source_module.replace('_', ' ')}
                                  {f.affected_path_or_endpoint && <span className="text-gray-600 ml-2">→ {f.affected_path_or_endpoint}</span>}
                                  
                                  {f.correlation_count > 0 && (
                                    <span className={clsx(
                                      "ml-1 px-1.5 py-0.5 rounded border text-[9px] uppercase font-bold tracking-widest flex items-center gap-1",
                                      isWeakContext ? "bg-gray-500/10 text-gray-500 border-gray-500/20" : "bg-accent-cyan/10 text-accent-cyan border-accent-cyan/20"
                                    )}>
                                      <Link className="w-2.5 h-2.5"/> 
                                      {isWeakContext ? `Consistent (${f.correlation_count})` : `Correlated (${f.correlation_count})`}
                                    </span>
                                  )}

                                  {/* Context adjustment badge */}
                                  {isElevated && (
                                    <span className="ml-1 px-1.5 py-0.5 bg-green-500/10 text-green-400 rounded border border-green-500/20 text-[9px] uppercase font-bold tracking-widest flex items-center gap-1">
                                      <ArrowUp className="w-2.5 h-2.5"/> Elevated
                                    </span>
                                  )}
                                  {isDowngraded && (
                                    <span className="ml-1 px-1.5 py-0.5 bg-yellow-500/10 text-yellow-500 rounded border border-yellow-500/20 text-[9px] uppercase font-bold tracking-widest flex items-center gap-1">
                                      <ArrowDown className="w-2.5 h-2.5"/> Limited Context
                                    </span>
                                  )}
                                  {isSuppressed && (
                                    <span className="ml-1 px-1.5 py-0.5 bg-gray-500/10 text-gray-500 rounded border border-gray-500/20 text-[9px] uppercase font-bold tracking-widest flex items-center gap-1">
                                      <EyeOff className="w-2.5 h-2.5"/> Low Exploitability
                                    </span>
                                  )}
                                  {f.correlation_is_hygiene_gap && (
                                    <span className="ml-1 px-1.5 py-0.5 bg-blue-500/10 text-blue-400 rounded border border-blue-500/20 text-[9px] uppercase font-bold tracking-widest flex items-center gap-1">
                                      <ShieldX className="w-2.5 h-2.5"/> Hygiene Gap
                                    </span>
                                  )}
                                </span>
                              </div>
                            </div>
                            <div className="shrink-0 ml-4">
                              {isExpanded ? <ChevronDown className="w-5 h-5 text-gray-500" /> : <ChevronRight className="w-5 h-5 text-gray-600" />}
                            </div>
                          </div>

                          {/* Expanded Details */}
                          <AnimatePresence>
                            {isExpanded && (
                              <motion.div 
                                initial={{ height: 0, opacity: 0 }} 
                                animate={{ height: 'auto', opacity: 1 }} 
                                exit={{ height: 0, opacity: 0 }}
                                className="overflow-hidden bg-[#0f0f13] border-t border-white/[0.02]"
                              >
                                <div className="p-6 grid grid-cols-1 lg:grid-cols-3 gap-8">
                                  {/* Left Col: Metadata */}
                                  <div className="col-span-1 space-y-5">
                                    <div>
                                      <p className="text-[10px] uppercase text-gray-500 tracking-widest font-bold mb-3">Context Metadata</p>
                                      <div className="space-y-2">
                                        <div className="flex justify-between items-center text-xs border-b border-white/5 pb-2">
                                          <span className="text-gray-500">Status</span>
                                          <span className={clsx("font-medium", f.status === 'open' ? 'text-red-400' : 'text-green-400')}>{f.status}</span>
                                        </div>
                                        <div className="flex justify-between items-center text-xs border-b border-white/5 pb-2">
                                          <span className="text-gray-500">Category</span>
                                          <span className="text-gray-300">{f.category}</span>
                                        </div>
                                        <div className="flex justify-between items-center text-xs border-b border-white/5 pb-2">
                                          <span className="text-gray-500">Exploitability</span>
                                          <span className={clsx("capitalize", f.exploitability === 'proven' ? 'text-red-400 font-bold' : 'text-gray-400 font-medium')}>{f.exploitability || "Unknown"}</span>
                                        </div>
                                        <div className="flex justify-between items-center text-xs border-b border-white/5 pb-2">
                                          <span className="text-gray-500">Evidence Wgt</span>
                                          <span className={clsx("capitalize", f.evidence_weight === 'strong' ? 'text-green-400 font-bold' : 'text-gray-400 font-medium')}>{f.evidence_weight || "Unknown"}</span>
                                        </div>
                                        {f.protocol && (
                                          <div className="flex justify-between items-center text-xs border-b border-white/5 pb-2">
                                            <span className="text-gray-500">Protocol</span>
                                            <span className="text-accent-cyan font-mono">{f.protocol}</span>
                                          </div>
                                        )}
                                        {f.method && (
                                          <div className="flex justify-between items-center text-xs border-b border-white/5 pb-2">
                                            <span className="text-gray-500">Method</span>
                                            <span className="text-gray-300 font-mono">{f.method}</span>
                                          </div>
                                        )}
                                      </div>
                                    </div>

                                    {f.correlation_summary && (
                                      <div>
                                        <p className="text-[10px] uppercase text-accent-cyan tracking-widest font-bold mb-2 flex items-center gap-1.5"><Link className="w-3 h-3"/> Correlation Insight</p>
                                        <p className="text-xs text-cyan-100 bg-accent-cyan/10 border border-accent-cyan/20 rounded p-3 leading-relaxed">
                                          {f.correlation_summary}
                                        </p>
                                      </div>
                                    )}

                                    {f.risk_score && (
                                      <div>
                                        <p className="text-[10px] uppercase text-amber-500 tracking-widest font-bold mb-2 flex items-center gap-1.5"><TrendingUp className="w-3 h-3"/> Why this severity was assigned</p>
                                        <div className="bg-amber-500/5 border border-amber-500/20 rounded-lg p-3 space-y-2">
                                          <p className="text-xs text-amber-100 font-medium flex items-center gap-2">
                                            <Zap className="w-3 h-3 text-amber-400"/>
                                            {f.risk_score.priority_statement}
                                          </p>

                                          {(f.risk_score.finding_score !== undefined || f.risk_score.correlation_score !== undefined) && (
                                            <div className="flex gap-4 text-[10px] font-mono text-amber-500/80 bg-amber-900/10 p-2 rounded">
                                              <span>Finding Score: {f.risk_score.finding_score}</span>
                                              {f.risk_score.correlation_score > 0 && (
                                                <span>Correlation Score: +{f.risk_score.correlation_score}</span>
                                              )}
                                              <span className="text-amber-400 font-bold">Total: {f.risk_score.total_score}</span>
                                            </div>
                                          )}

                                          {f.risk_score.escalation_reason && (
                                            <div className={clsx(
                                              "text-[10px] p-2 rounded font-medium flex items-start gap-2 mt-2 border",
                                              f.risk_score.escalation_reason.includes("escalated") ? "bg-red-500/10 border-red-500/20 text-red-300" :
                                              "bg-gray-500/10 border-gray-500/20 text-gray-400"
                                            )}>
                                              {f.risk_score.escalation_reason.includes("escalated") ? <ArrowUp className="w-3 h-3 mt-0.5 shrink-0"/> : <ShieldX className="w-3 h-3 mt-0.5 shrink-0"/>}
                                              <span>{f.risk_score.escalation_reason}</span>
                                            </div>
                                          )}

                                          <div className="space-y-1 pt-2 border-t border-amber-500/10">
                                            {f.risk_score.contributions?.map((c: any, ci: number) => (
                                              <div key={ci} className="flex items-center gap-2 text-[10px] font-mono">
                                                <span className={c.delta >= 0 ? 'text-red-400' : 'text-green-400'}>
                                                  {c.delta >= 0 ? `+${c.delta}` : c.delta}
                                                </span>
                                                <span className="text-gray-400">{c.explanation}</span>
                                              </div>
                                            ))}
                                          </div>
                                        </div>
                                      </div>
                                    )}

                                    {/* Phase 5: Context-Aware Assessment */}
                                    {f.context_assessment && (
                                      <div>
                                        <p className="text-[10px] uppercase text-purple-400 tracking-widest font-bold mb-2 flex items-center gap-1.5">
                                          <ShieldX className="w-3 h-3"/> {(isDowngraded || isSuppressed || f.correlation_is_hygiene_gap) ? "Why this is not higher severity" : "Context Assessment"}
                                        </p>
                                        <div className={clsx(
                                          "rounded-lg p-3 space-y-2 border",
                                          f.context_assessment.adjustment === 'elevated' ? 'bg-green-500/5 border-green-500/20' :
                                          f.context_assessment.adjustment === 'downgraded' ? 'bg-yellow-500/5 border-yellow-500/20' :
                                          f.context_assessment.adjustment === 'suppressed' ? 'bg-gray-500/5 border-gray-500/20' :
                                          'bg-purple-500/5 border-purple-500/20'
                                        )}>
                                          <p className={clsx(
                                            "text-xs font-medium flex items-center gap-2",
                                            f.context_assessment.adjustment === 'elevated' ? 'text-green-300' :
                                            f.context_assessment.adjustment === 'downgraded' ? 'text-yellow-300' :
                                            f.context_assessment.adjustment === 'suppressed' ? 'text-gray-400' :
                                            'text-purple-300'
                                          )}>
                                            {f.context_assessment.adjustment === 'elevated' ? <ArrowUp className="w-3 h-3"/> :
                                             f.context_assessment.adjustment === 'downgraded' ? <ArrowDown className="w-3 h-3"/> :
                                             f.context_assessment.adjustment === 'suppressed' ? <EyeOff className="w-3 h-3"/> :
                                             <Minus className="w-3 h-3"/>}
                                            {f.context_assessment.context_summary}
                                          </p>

                                          {f.context_assessment.score_delta !== 0 && (
                                            <div className="text-[10px] font-mono text-gray-400 pt-1 border-t border-white/5 flex items-center gap-3">
                                              <span>Original: {f.risk_score?.total_score ?? '?'}</span>
                                              <span className={f.context_assessment.score_delta > 0 ? 'text-green-400' : 'text-yellow-400'}>
                                                {f.context_assessment.score_delta > 0 ? `+${f.context_assessment.score_delta}` : f.context_assessment.score_delta}
                                              </span>
                                              <span>→ Adjusted: {f.context_assessment.adjusted_score}</span>
                                            </div>
                                          )}

                                          {f.context_assessment.signals?.length > 0 && (
                                            <div className="flex flex-wrap gap-1 pt-1">
                                              {f.context_assessment.signals.map((s: string, si: number) => (
                                                <span key={si} className="px-1.5 py-0.5 bg-green-500/10 text-green-400 border border-green-500/20 rounded text-[8px] uppercase tracking-widest font-bold">
                                                  {s.replace(/_/g, ' ')}
                                                </span>
                                              ))}
                                            </div>
                                          )}

                                          {f.context_assessment.noise_indicators?.length > 0 && (
                                            <div className="flex flex-wrap gap-1 pt-1">
                                              {f.context_assessment.noise_indicators.map((n: string, ni: number) => (
                                                <span key={ni} className="px-1.5 py-0.5 bg-yellow-500/10 text-yellow-500 border border-yellow-500/20 rounded text-[8px] uppercase tracking-widest font-bold">
                                                  {n.replace(/_/g, ' ')}
                                                </span>
                                              ))}
                                            </div>
                                          )}
                                        </div>
                                      </div>
                                    )}

                                    {f.raw_reference && (
                                      <div>
                                        <p className="text-[10px] uppercase text-gray-500 tracking-widest font-bold mb-2 flex items-center gap-1.5"><Fingerprint className="w-3 h-3"/> Raw Trace</p>
                                        <div className="bg-black/50 border border-white/5 rounded overflow-hidden">
                                           <pre className="p-3 text-[9px] font-mono text-gray-500 overflow-x-auto max-h-32 custom-scrollbar">
                                             {JSON.stringify(f.raw_reference, null, 2)}
                                            </pre>
                                        </div>
                                      </div>
                                    )}
                                  </div>
                                  
                                  {/* Right Col: Evidence */}
                                  <div className="col-span-1 lg:col-span-2 space-y-5">
                                    {f.technical_details && (
                                      <div>
                                        <p className="text-[10px] uppercase text-gray-500 tracking-widest font-bold mb-2 flex items-center gap-1.5"><Info className="w-3 h-3"/> Technical Detail</p>
                                        <p className="text-xs text-gray-300 bg-white/[0.02] p-4 rounded-lg font-mono leading-relaxed border border-white/5 whitespace-pre-wrap">{f.technical_details}</p>
                                      </div>
                                    )}
                                    
                                    {f.evidence?.length > 0 && (
                                      <div>
                                        <p className="text-[10px] uppercase text-gray-500 tracking-widest font-bold mb-2 flex items-center gap-1.5"><Code className="w-3 h-3"/> Concrete Evidence</p>
                                        <div className="space-y-3">
                                          {f.evidence.map((ev: any, j: number) => (
                                            <div key={j} className="bg-black border border-white/5 rounded-lg overflow-hidden flex flex-col">
                                              <div className="bg-white/5 px-3 py-2 border-b border-white/5 flex justify-between items-center">
                                                <span className="text-[10px] text-accent-cyan uppercase tracking-wider font-semibold">
                                                  {ev.description}
                                                </span>
                                                {ev.validation_context && (
                                                  <span className="text-[10px] text-gray-500 font-mono">
                                                    Check: {ev.validation_context}
                                                  </span>
                                                )}
                                              </div>
                                              <pre className="p-3 text-[10px] font-mono text-green-400 overflow-x-auto max-h-40 custom-scrollbar whitespace-pre-wrap leading-relaxed">
                                                {ev.raw_data}
                                              </pre>
                                            </div>
                                          ))}
                                        </div>
                                      </div>
                                    )}

                                    {f.related_findings?.length > 0 && (
                                      <div>
                                        <p className="text-[10px] uppercase text-gray-500 tracking-widest font-bold mb-2 flex items-center gap-1.5"><Link className="w-3 h-3"/> Siblings in Cluster</p>
                                        <div className="space-y-2">
                                          {f.related_findings.map((rel: any, k: number) => (
                                            <div key={k} className="bg-white/[0.02] border border-white/5 rounded p-2 flex items-center gap-3">
                                              <span className="px-1.5 py-0.5 rounded text-[9px] uppercase font-bold tracking-widest bg-gray-500/20 text-gray-400 border border-gray-500/30">
                                                {rel.severity}
                                              </span>
                                              <span className="text-xs text-gray-300 font-medium truncate flex-1">{rel.short_summary}</span>
                                              <span className="text-[10px] text-gray-500 font-mono hidden sm:block">{rel.category}</span>
                                            </div>
                                          ))}
                                        </div>
                                      </div>
                                    )}
                                  </div>
                                </div>
                              </motion.div>
                            )}
                          </AnimatePresence>
                        </div>
                      );
                    })}
                  </div>
                  </>
                )}
            </motion.div>
          )}

          {/* LOGS */}
          {activeTab === 'logs' && (
            <motion.div key="logs" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="bg-[#050505] p-6 relative h-[500px]">
              <div className="absolute top-0 right-0 p-4 font-mono text-[10px] text-gray-600 opacity-50 z-0 select-none">LIMMA_AUDIT_STDOUT</div>
              <div className="overflow-y-auto h-full space-y-1.5 custom-scrollbar relative z-10 font-mono text-[11px] pr-4">
                {auditData.normalization_log?.map((log: string, i: number) => {
                  const isHighlighted = log.includes('Started') || log.includes('Extracted') || log.includes('Generated');
                  return (
                    <div key={i} className="flex gap-3 hover:bg-white/[0.02] transition-colors px-2 py-0.5 rounded leading-relaxed">
                      <span className="text-gray-600 select-none opacity-50 flex-shrink-0">{(i + 1).toString().padStart(3, '0')}</span>
                      <span className="text-accent-cyan/50 select-none flex-shrink-0">{'>'}</span>
                      <span className={clsx(
                        isHighlighted ? "text-gray-300 font-medium" : "text-gray-500",
                        log.includes('Normalizer') && "text-purple-400"
                      )}>
                        {log}
                      </span>
                    </div>
                  );
                })}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}
