"use client";

import { useState } from "react";
import { useForm, Controller } from "react-hook-form";
import { cn } from "@/shared/lib/utils";
import { runBlindScan } from "@/features/blind-scan/api/blind-scan-api";
import {
  Eye, Loader2, Play, AlertTriangle, CheckCircle, ChevronDown, Crosshair, Clock,
  Shield, XCircle, Bug, Zap,
} from "lucide-react";

/** Safe JSON stringifier that handles circular references */
function getCircularReplacer() {
  const seen = new WeakSet();
  return (_key: string, value: unknown) => {
    if (typeof value === "object" && value !== null) {
      if (seen.has(value)) return "[Circular]";
      seen.add(value);
    }
    return value;
  };
}

const DETECTION_TYPES = [
  "dom_xss",
  "blind_sqli_boolean",
  "blind_sqli_time_based",
  "blind_sqli_error_based",
  "blind_ssrf_dns",
  "blind_ssrf_http",
  "second_order_injection",
  "race_condition",
  "jwt_none_alg",
  "xml_external_entity",
  "insecure_deserialization",
];

const TYPE_LABELS: Record<string, string> = {
  dom_xss: "DOM XSS",
  blind_sqli_boolean: "SQLi Boolean",
  blind_sqli_time_based: "SQLi Time-Based",
  blind_sqli_error_based: "SQLi Error-Based",
  blind_ssrf_dns: "SSRF DNS",
  blind_ssrf_http: "SSRF HTTP",
  second_order_injection: "2nd Order Injection",
  race_condition: "Race Condition",
  jwt_none_alg: "JWT None Alg",
  xml_external_entity: "XXE",
  insecure_deserialization: "Deserialization",
};

interface BlindScanForm {
  targetUrl: string;
  selectedTypes: string[];
  maxDuration: number;
}

/* ── Result Parsing Helpers ── */
interface ParsedFinding {
  type: string;
  title: string;
  severity: string;
  vulnerable: boolean;
  confidence: number | string;
  url?: string;
  parameter?: string;
  payload?: string;
  evidence?: string;
  description?: string;
  cwe?: string;
  cvss?: number;
  details?: Record<string, unknown>;
}

function parseSeverity(s: unknown): string {
  if (typeof s === "string") return s.toLowerCase();
  return "info";
}

function parseFindings(raw: Record<string, unknown>): ParsedFinding[] {
  const findings: ParsedFinding[] = [];

  // Pattern 1: { results: [{ vuln_type, vulnerable, ... }] }
  if (Array.isArray(raw.results)) {
    for (const r of raw.results) {
      if (typeof r !== "object" || !r) continue;
      const item = r as Record<string, unknown>;
      findings.push({
        type: String(item.vuln_type ?? item.detection_type ?? item.type ?? "unknown"),
        title: String(item.title ?? item.name ?? item.vuln_type ?? "Finding"),
        severity: parseSeverity(item.severity ?? item.risk_level),
        vulnerable: Boolean(item.vulnerable ?? item.is_vulnerable ?? item.found),
        confidence: (item.confidence as number) ?? (item.confidence_score as number) ?? "—",
        url: item.url as string | undefined,
        parameter: item.parameter as string | undefined,
        payload: item.payload as string | undefined,
        evidence: typeof item.evidence === "string" ? item.evidence : (item.evidence ? JSON.stringify(item.evidence, null, 2) : undefined),
        description: item.description as string | undefined,
        cwe: item.cwe as string | undefined,
        cvss: item.cvss as number | undefined,
        details: item,
      });
    }
    return findings;
  }

  // Pattern 2: { findings: [...] }
  if (Array.isArray(raw.findings)) {
    for (const r of raw.findings) {
      if (typeof r !== "object" || !r) continue;
      const item = r as Record<string, unknown>;
      findings.push({
        type: String(item.vuln_type ?? item.detection_type ?? item.type ?? "unknown"),
        title: String(item.title ?? item.name ?? "Finding"),
        severity: parseSeverity(item.severity),
        vulnerable: Boolean(item.vulnerable ?? item.is_vulnerable ?? true),
        confidence: (item.confidence as number) ?? "—",
        url: item.url as string | undefined,
        parameter: item.parameter as string | undefined,
        payload: item.payload as string | undefined,
        evidence: typeof item.evidence === "string" ? item.evidence : (item.evidence ? JSON.stringify(item.evidence, null, 2) : undefined),
        description: item.description as string | undefined,
        cwe: item.cwe as string | undefined,
        cvss: item.cvss as number | undefined,
        details: item,
      });
    }
    return findings;
  }

  // Pattern 3: Each detection type is a key with result
  for (const [key, val] of Object.entries(raw)) {
    if (key === "scan_id" || key === "target_url" || key === "duration" || key === "status" || key === "timestamp" || key === "scan_duration_ms" || key === "total_checks") continue;
    if (typeof val === "object" && val !== null && !Array.isArray(val)) {
      const item = val as Record<string, unknown>;
      findings.push({
        type: key,
        title: String(item.title ?? TYPE_LABELS[key] ?? key),
        severity: parseSeverity(item.severity ?? item.risk_level),
        vulnerable: Boolean(item.vulnerable ?? item.is_vulnerable ?? item.found ?? false),
        confidence: (item.confidence as number) ?? "—",
        url: item.url as string | undefined,
        parameter: item.parameter as string | undefined,
        payload: item.payload as string | undefined,
        evidence: typeof item.evidence === "string" ? item.evidence : undefined,
        description: item.description as string | undefined,
        details: item,
      });
    }
  }

  return findings;
}

function getSeverityStyle(sev: string) {
  switch (sev) {
    case "critical": return { border: "border-critical/30", bg: "bg-critical/10", text: "text-critical" };
    case "high": return { border: "border-high/30", bg: "bg-high/10", text: "text-high" };
    case "medium": return { border: "border-attention/30", bg: "bg-attention/10", text: "text-attention" };
    case "low": return { border: "border-primary/30", bg: "bg-primary/10", text: "text-primary" };
    default: return { border: "border-border/30", bg: "bg-muted/10", text: "text-muted-foreground" };
  }
}

/** Convert snake_case keys to readable Title Case */
function formatKeyLabel(key: string): string {
  return key
    .replace(/_/g, " ")
    .replace(/\b\w/g, c => c.toUpperCase())
    .replace(/\bUrl\b/g, "URL")
    .replace(/\bId\b/g, "ID")
    .replace(/\bSsrf\b/g, "SSRF")
    .replace(/\bXss\b/g, "XSS")
    .replace(/\bSqli\b/g, "SQLi")
    .replace(/\bCwe\b/g, "CWE")
    .replace(/\bCvss\b/g, "CVSS")
    .replace(/\bMs\b/g, "ms")
    .replace(/\bDns\b/g, "DNS")
    .replace(/\bHttp\b/g, "HTTP")
    .replace(/\bJwt\b/g, "JWT");
}

/** Format a raw value for display */
function formatValue(value: unknown): { display: string; style: string; isBool?: boolean; boolVal?: boolean } {
  if (value === null || value === undefined) return { display: "—", style: "text-muted-foreground/30 italic" };
  if (typeof value === "boolean") return { display: value ? "TRUE" : "FALSE", style: "", isBool: true, boolVal: value };
  if (typeof value === "number") {
    // Format large decimals
    if (!Number.isInteger(value) && String(value).length > 8) return { display: value.toFixed(4), style: "font-mono font-bold text-primary tabular-nums" };
    return { display: String(value), style: "font-mono font-bold text-primary tabular-nums" };
  }
  if (typeof value === "string") {
    // UUID detection
    if (/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(value)) {
      return { display: value, style: "font-mono text-muted-foreground/60 text-[9px]" };
    }
    // ISO date
    if (/^\d{4}-\d{2}-\d{2}T/.test(value)) {
      try { return { display: new Date(value).toLocaleString("tr-TR"), style: "text-foreground/70" }; } catch { /* fallback */ }
    }
    // URL
    if (value.startsWith("http")) return { display: value, style: "font-mono text-primary break-all" };
    // SQL / payload-like
    if (/['";(){}]/.test(value) && value.length < 200) return { display: value, style: "font-mono text-attention bg-attention/5 px-1.5 py-0.5 rounded" };
    return { display: value, style: "text-foreground/80" };
  }
  if (Array.isArray(value)) {
    if (value.length === 0) return { display: "None", style: "text-muted-foreground/30 italic" };
    return { display: value.map(v => typeof v === "object" ? JSON.stringify(v) : String(v)).join(", "), style: "text-foreground/70" };
  }
  return { display: JSON.stringify(value), style: "text-muted-foreground/50 font-mono text-[9px]" };
}

/** Recursively render all details as structured key-value table */
function DetailRenderer({ data, depth = 0 }: { data: Record<string, unknown>; depth?: number }) {
  const entries = Object.entries(data);
  if (entries.length === 0) return <span className="text-muted-foreground/40 italic text-[10px]">No data</span>;

  // Separate nested objects from flat values
  const flatEntries = entries.filter(([, v]) => typeof v !== "object" || v === null || Array.isArray(v));
  const nestedEntries = entries.filter(([, v]) => typeof v === "object" && v !== null && !Array.isArray(v));

  return (
    <div className="space-y-3">
      {/* Flat key-value pairs as aligned grid */}
      {flatEntries.length > 0 && (
        <div className="grid gap-y-1" style={{ gridTemplateColumns: "140px 1fr" }}>
          {flatEntries.map(([key, val]) => {
            const label = formatKeyLabel(key);
            const formatted = formatValue(val);
            return (
              <div key={key} className="contents">
                <div className="text-[10px] text-muted-foreground/50 font-medium py-0.5 flex items-center gap-1.5">
                  <span className="w-1 h-1 rounded-full bg-primary/40 shrink-0" />
                  {label}
                </div>
                <div className="text-[11px] py-0.5">
                  {formatted.isBool ? (
                    <span className={cn(
                      "inline-flex items-center gap-1 text-[9px] font-bold uppercase tracking-wider px-2 py-0.5 rounded border",
                      formatted.boolVal
                        ? "bg-verified/10 text-verified border-verified/20"
                        : "bg-risk/10 text-risk border-risk/20"
                    )}>
                      {formatted.boolVal ? <CheckCircle className="h-2.5 w-2.5" /> : <XCircle className="h-2.5 w-2.5" />}
                      {formatted.display}
                    </span>
                  ) : (
                    <span className={formatted.style}>{formatted.display}</span>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Nested objects as sub-sections */}
      {nestedEntries.map(([key, val]) => {
        const label = formatKeyLabel(key);
        return (
          <div key={key} className="space-y-2">
            <div className="flex items-center gap-2">
              <Shield className="h-3 w-3 text-primary/60" />
              <span className="text-[10px] text-primary font-bold uppercase tracking-widest">{label}</span>
              <div className="flex-1 h-px bg-border/15" />
            </div>
            <div className={cn("rounded-md border border-border/10 bg-[#060606] p-3", depth > 0 && "ml-2")}>
              <DetailRenderer data={val as Record<string, unknown>} depth={depth + 1} />
            </div>
          </div>
        );
      })}
    </div>
  );
}


/* ── Component ── */
export function BlindScannerScreen() {
  const { register, control, handleSubmit, formState: { errors } } = useForm<BlindScanForm>({
    defaultValues: {
      targetUrl: "",
      selectedTypes: DETECTION_TYPES,
      maxDuration: 120,
    }
  });

  const [isPending, setIsPending] = useState(false);
  const [result, setResult] = useState<Record<string, unknown> | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [showConfig, setShowConfig] = useState(true);
  const [expandedIdx, setExpandedIdx] = useState<number | null>(null);

  const onSubmit = async (data: BlindScanForm) => {
    setIsPending(true);
    setError(null);
    setResult(null);
    try {
      const res = await runBlindScan({
        target_url: data.targetUrl,
        detection_types: data.selectedTypes,
        max_duration_seconds: data.maxDuration,
      });
      setResult(res);
      setShowConfig(false); // collapse config on success
    } catch (err) {
      setError(err instanceof Error ? err.message : "Blind scan failed");
    } finally {
      setIsPending(false);
    }
  };

  const parsedFindings = result ? parseFindings(result) : [];
  const vulnCount = parsedFindings.filter(f => f.vulnerable).length;
  const safeCount = parsedFindings.filter(f => !f.vulnerable).length;

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden">
      {/* Top bar */}
      <div className="shrink-0 flex items-center justify-between gap-4 px-4 py-2.5 border-b border-border/40 bg-[#050505]">
        <div className="flex items-center gap-1.5">
          <Eye className="h-4 w-4 text-primary" />
          <h2 className="text-[13px] font-bold tracking-tight text-foreground">Blind Scanner</h2>
        </div>
        {isPending && (
          <div className="flex items-center gap-1.5 text-[10px] text-primary animate-pulse font-mono">
            <Loader2 className="h-3 w-3 animate-spin" /> Running blind scan…
          </div>
        )}
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4 bg-[#030303]">
        {/* Config panel */}
        <div className="relative pl-6">
          <div className="absolute top-3 bottom-3 left-2.5 w-[2px] bg-gradient-to-b from-primary via-primary/40 to-transparent" />

          <div className="relative">
            <div className={cn(
              "absolute -left-5 top-4 w-3 h-3 rounded-full border-2 border-background z-10 transition-transform duration-300",
              "bg-primary shadow-[0_0_8px_var(--primary)]",
              showConfig && "scale-125"
            )} />

            <div className={cn(
              "bg-[#080808] border rounded-md shadow-lg transition-colors duration-300 overflow-hidden ml-2",
              showConfig ? "border-primary/40" : "border-border/20 hover:border-border/50"
            )}>
              <div
                className="p-3.5 cursor-pointer flex items-center justify-between gap-3 hover:bg-white/[0.02]"
                onClick={() => setShowConfig(!showConfig)}
              >
                <div className="flex items-center gap-3">
                  <Crosshair className={cn("h-4 w-4", showConfig ? "text-primary drop-shadow-[0_0_4px_var(--primary)]" : "text-muted-foreground/60")} />
                  <span className={cn("text-[12px] font-bold uppercase tracking-wider transition-colors", showConfig ? "text-primary" : "text-foreground/90")}>
                    Scan Configuration
                  </span>
                </div>
                <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-300", showConfig && "rotate-180")} />
              </div>

              <div className={cn("grid transition-all duration-300 ease-in-out", showConfig ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0")}>
                <div className="overflow-hidden">
                  <form className="px-4 pb-4 pt-2 border-t border-border/10 bg-black/40 space-y-4" onSubmit={handleSubmit(onSubmit)}>
                    <div className="space-y-1.5">
                      <label className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">Target URL</label>
                      <input
                        type="text"
                        placeholder="https://example.com"
                        {...register("targetUrl", { required: "Target URL is required" })}
                        className={cn(
                          "w-full h-9 px-3 text-[11px] font-mono bg-[#0c0c0c] border rounded-md text-foreground placeholder:text-muted-foreground/30 focus:outline-none focus:ring-1 transition-all",
                          errors.targetUrl ? "border-risk/50 focus:border-risk focus:ring-risk/20" : "border-border/30 focus:border-primary/50 focus:ring-primary/20"
                        )}
                      />
                      {errors.targetUrl && <span className="text-risk text-[9px] font-mono">{errors.targetUrl.message}</span>}
                    </div>

                    <div className="space-y-2">
                      <label className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">Detection Types</label>
                      <Controller
                        name="selectedTypes"
                        control={control}
                        rules={{ validate: val => val.length > 0 || "Select at least one detection type" }}
                        render={({ field }) => (
                          <div className="flex flex-wrap gap-1.5">
                            {DETECTION_TYPES.map((type) => {
                              const isSelected = field.value.includes(type);
                              return (
                                <button
                                  key={type}
                                  type="button"
                                  className={cn(
                                    "px-2.5 py-1 rounded text-[9px] font-bold uppercase tracking-wider transition-all duration-200 border",
                                    isSelected
                                      ? "bg-primary/10 border-primary/30 text-primary shadow-[0_0_6px_var(--primary)/10]"
                                      : "bg-[#0c0c0c] border-border/20 text-muted-foreground/50 hover:border-border/50 hover:text-muted-foreground"
                                  )}
                                  onClick={() => {
                                    const newVal = isSelected 
                                      ? field.value.filter(t => t !== type)
                                      : [...field.value, type];
                                    field.onChange(newVal);
                                  }}
                                >
                                  {TYPE_LABELS[type] ?? type}
                                </button>
                              );
                            })}
                          </div>
                        )}
                      />
                      {errors.selectedTypes && <span className="text-risk text-[9px] font-mono">{errors.selectedTypes.message}</span>}
                    </div>

                    <div className="flex items-center gap-3">
                      <Clock className="h-3.5 w-3.5 text-muted-foreground/40" />
                      <label className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">Max Duration (s)</label>
                      <input
                        type="number"
                        {...register("maxDuration", { valueAsNumber: true, min: 1 })}
                        className="w-20 h-8 px-2 text-[11px] font-mono bg-[#0c0c0c] border border-border/30 rounded-md text-center text-foreground focus:outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/20 transition-all"
                      />
                    </div>

                    <button
                      type="submit"
                      className={cn(
                        "flex items-center gap-2 px-5 py-2.5 rounded-md text-[11px] font-bold uppercase tracking-wider transition-all duration-200 border",
                        isPending
                          ? "bg-primary/5 text-primary/60 border-primary/20 cursor-wait"
                          : "bg-primary/10 text-primary border-primary/30 hover:bg-primary/20 hover:shadow-[0_0_12px_var(--primary)/15]"
                      )}
                      disabled={isPending}
                    >
                      {isPending ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Play className="h-3.5 w-3.5" />}
                      {isPending ? "Running Blind Scan…" : "Start Blind Scan"}
                    </button>
                  </form>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Error */}
        {error && (
          <div className="flex items-center gap-2 text-[11px] text-risk bg-risk/5 border border-risk/10 rounded-md px-4 py-2.5 font-mono">
            <AlertTriangle className="h-3.5 w-3.5 shrink-0" /> {error}
          </div>
        )}

        {/* Results */}
        {result && (
          <div className="space-y-4">
            {/* Results summary HUD */}
            <div className="flex flex-wrap items-center gap-3 p-3 bg-[#080808] rounded-md border border-border/30 shadow-inner">
              <div className="flex items-center gap-2">
                <Shield className="h-4 w-4 text-primary" />
                <span className="text-[11px] font-bold text-foreground uppercase tracking-widest">Scan Results</span>
              </div>
              <div className="flex items-center gap-3 ml-auto">
                {vulnCount > 0 && (
                  <div className="flex items-baseline gap-1.5 bg-risk/10 border border-risk/20 px-2.5 py-1 rounded shadow-sm">
                    <Bug className="h-3 w-3 text-risk self-center" />
                    <span className="text-[15px] font-mono font-bold text-risk">{vulnCount}</span>
                    <span className="text-[9px] uppercase tracking-widest text-risk/80 font-bold">Vulnerable</span>
                  </div>
                )}
                <div className="flex items-baseline gap-1.5 bg-verified/10 border border-verified/20 px-2.5 py-1 rounded shadow-sm">
                  <CheckCircle className="h-3 w-3 text-verified self-center" />
                  <span className="text-[15px] font-mono font-bold text-verified">{safeCount}</span>
                  <span className="text-[9px] uppercase tracking-widest text-verified/80 font-bold">Safe</span>
                </div>
                <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded">
                  <span className="text-[15px] font-mono font-bold text-primary">{parsedFindings.length}</span>
                  <span className="text-[9px] uppercase tracking-widest text-muted-foreground/50 font-bold">Total</span>
                </div>
                {/* Meta from response */}
                {result.scan_duration_ms != null && (
                  <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded">
                    <Clock className="h-3 w-3 text-muted-foreground/50 self-center" />
                    <span className="text-[11px] font-mono font-bold text-foreground">{String(result.scan_duration_ms)}ms</span>
                  </div>
                )}
                {result.duration != null && (
                  <div className="flex items-baseline gap-1.5 bg-[#0a0a0a] border border-border/20 px-2.5 py-1 rounded">
                    <Clock className="h-3 w-3 text-muted-foreground/50 self-center" />
                    <span className="text-[11px] font-mono font-bold text-foreground">{String(result.duration)}</span>
                  </div>
                )}
              </div>
            </div>

            {/* Parsed findings chain */}
            {parsedFindings.length > 0 ? (
              <div className="relative pl-6 space-y-2">
                <div className="absolute top-3 bottom-3 left-2.5 w-[2px] bg-gradient-to-b from-primary via-primary/40 to-transparent" />

                {parsedFindings.map((f, i) => {
                  const isExpanded = expandedIdx === i;
                  const sevStyle = getSeverityStyle(f.severity);

                  return (
                    <div key={i} className="relative group">
                      <div className={cn(
                        "absolute -left-5 top-4 w-3 h-3 rounded-full border-2 border-background z-10 transition-transform duration-300",
                        f.vulnerable
                          ? "bg-risk shadow-[0_0_8px_var(--risk)]"
                          : "bg-verified shadow-[0_0_8px_var(--verified)]",
                        isExpanded && "scale-125"
                      )} />

                      <div className={cn(
                        "bg-[#080808] border rounded-md shadow-lg transition-colors duration-300 overflow-hidden ml-2",
                        isExpanded ? (f.vulnerable ? "border-risk/40" : "border-verified/40") : "border-border/20 hover:border-border/50"
                      )}>
                        <div
                          className="p-3.5 cursor-pointer flex items-center justify-between gap-3 hover:bg-white/[0.02]"
                          onClick={() => setExpandedIdx(isExpanded ? null : i)}
                        >
                          <div className="flex items-center gap-3 min-w-0">
                            {f.vulnerable ? (
                              <Bug className="h-4 w-4 text-risk shrink-0" />
                            ) : (
                              <CheckCircle className="h-4 w-4 text-verified shrink-0" />
                            )}
                            <span className={cn(
                              "text-[9px] font-bold uppercase tracking-wider px-2 py-0.5 rounded border shrink-0",
                              f.vulnerable ? "bg-risk/10 text-risk border-risk/20" : "bg-verified/10 text-verified border-verified/20"
                            )}>
                              {f.vulnerable ? "VULN" : "SAFE"}
                            </span>
                            <span className={cn("text-[12px] font-bold truncate transition-colors", isExpanded ? "text-primary" : "text-foreground/90")}>
                              {TYPE_LABELS[f.type] ?? f.title}
                            </span>
                          </div>
                          <div className="flex items-center gap-3 shrink-0">
                            {f.severity !== "info" && (
                              <span className={cn("text-[9px] font-bold uppercase tracking-wider px-1.5 py-0.5 rounded border", sevStyle.bg, sevStyle.text, sevStyle.border)}>
                                {f.severity}
                              </span>
                            )}
                            {f.confidence !== "—" && (
                              <span className="text-[10px] font-mono font-bold text-muted-foreground/70 tabular-nums">
                                {typeof f.confidence === "number" ? `${f.confidence}%` : f.confidence}
                              </span>
                            )}
                            <ChevronDown className={cn("h-4 w-4 text-muted-foreground transition-transform duration-300", isExpanded && "rotate-180")} />
                          </div>
                        </div>

                        {/* Expanded details */}
                        <div className={cn("grid transition-all duration-300 ease-in-out", isExpanded ? "grid-rows-[1fr] opacity-100" : "grid-rows-[0fr] opacity-0")}>
                          <div className="overflow-hidden">
                            <div className="px-4 pb-4 pt-2 border-t border-border/10 bg-black/40 space-y-3">
                              {f.description && (
                                <p className="text-[11px] text-muted-foreground/80 leading-relaxed">{f.description}</p>
                              )}

                              {/* Payload */}
                              {f.payload && (
                                <div className="space-y-1.5">
                                  <span className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">Payload</span>
                                  <pre className="text-[10px] font-mono bg-[#0c0c0c] border border-border/15 rounded px-3 py-2 overflow-x-auto whitespace-pre-wrap break-all text-foreground/80 max-h-[150px] leading-relaxed">
                                    {f.payload}
                                  </pre>
                                </div>
                              )}

                              {/* Evidence */}
                              {f.evidence && (
                                <div className="space-y-1.5">
                                  <span className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">Evidence</span>
                                  <pre className="text-[10px] font-mono bg-[#0c0c0c] border border-border/15 rounded px-3 py-2 overflow-x-auto whitespace-pre-wrap break-all text-muted-foreground/70 max-h-[150px] leading-relaxed">
                                    {f.evidence}
                                  </pre>
                                </div>
                              )}

                              {/* Structured details — always render all details as readable rows */}
                              {f.details && (
                                <div className="space-y-1.5">
                                  <span className="text-[9px] text-muted-foreground/50 uppercase font-bold tracking-widest">Details</span>
                                  <div className="bg-[#0c0c0c] border border-border/15 rounded p-3 space-y-2">
                                    <DetailRenderer data={f.details} />
                                  </div>
                                </div>
                              )}
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>
            ) : (
              /* Fallback: if parsing found nothing, show raw in a styled box */
              <div className="bg-[#080808] border border-border/30 rounded-md shadow-lg overflow-hidden">
                <div className="px-4 py-3 border-b border-border/20 bg-gradient-to-r from-primary/10 to-transparent flex items-center gap-2">
                  <Zap className="h-4 w-4 text-primary" />
                  <span className="text-[13px] font-bold tracking-wide text-foreground">Raw Response</span>
                </div>
                <pre className="text-[10px] font-mono bg-black/40 px-4 py-3 overflow-x-auto max-h-[400px] overflow-y-auto whitespace-pre-wrap break-all text-muted-foreground/80 leading-relaxed">
                  {JSON.stringify(result, getCircularReplacer(), 2)}
                </pre>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
