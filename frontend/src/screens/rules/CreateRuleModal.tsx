import React, { useState } from "react";
import { X, Save, AlertCircle, RefreshCw } from "lucide-react";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/shared/ui/dialog";
import { cn } from "@/shared/lib/utils";
import { stringify } from "yaml";

interface CreateRuleModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (payload: { id: string; name: string; yaml_content: string }) => void;
  isSubmitting: boolean;
}

type RuleCategory =
  | "injection"
  | "xss"
  | "ssrf"
  | "authentication"
  | "misconfiguration"
  | "information_disclosure";

type RuleSeverity = "critical" | "high" | "medium" | "low" | "informational";
type RuleConfidence = "tentative";
type ConditionType = "body_contains" | "header_missing" | "header_present" | "status_code_in";

type RuleCondition =
  | { body_contains: { value: string } }
  | { header_missing: { header: string } }
  | { header_present: { header: string } }
  | { status_code_in: { codes: number[] } };

interface CustomRule {
  id: string;
  name: string;
  description: string;
  category: RuleCategory;
  severity: RuleSeverity;
  confidence: RuleConfidence;
  source: "user-custom";
  pack: "custom";
  condition: RuleCondition;
}

const HEADER_NAME_RE = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/;

function buildRuleId(name: string): string {
  const ruleId = name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/(^-|-$)+/g, "");

  return ruleId ? `custom-${ruleId}` : "custom-rule-id";
}

function parseStatusCodes(value: string): { codes: number[] } | { error: string } {
  const parts = value.split(",").map((part) => part.trim());
  const codes: number[] = [];

  for (const part of parts) {
    if (!part) {
      return { error: "Status codes must be a comma-separated list of numbers." };
    }

    if (!/^\d+$/.test(part)) {
      return { error: "Status codes can only contain numbers and commas." };
    }

    const code = Number(part);
    if (!Number.isInteger(code) || code < 100 || code > 599) {
      return { error: "Status codes must be valid HTTP codes between 100 and 599." };
    }

    codes.push(code);
  }

  return { codes: Array.from(new Set(codes)) };
}

function createCondition(
  conditionType: ConditionType,
  conditionValue: string,
  requireValue: boolean
): { condition: RuleCondition } | { error: string } {
  const trimmedValue = conditionValue.trim();

  if (requireValue && !trimmedValue) {
    return { error: "Condition value is required." };
  }

  if (/[\r\n]/.test(conditionValue)) {
    return { error: "Condition values must be a single line." };
  }

  if (!trimmedValue) {
    if (conditionType === "status_code_in") return { condition: { status_code_in: { codes: [] } } };
    if (conditionType === "header_missing") return { condition: { header_missing: { header: "" } } };
    if (conditionType === "header_present") return { condition: { header_present: { header: "" } } };
    return { condition: { body_contains: { value: "" } } };
  }

  if (conditionType === "status_code_in") {
    const parsed = parseStatusCodes(trimmedValue);
    if ("error" in parsed) return parsed;
    return { condition: { status_code_in: { codes: parsed.codes } } };
  }

  if (conditionType === "header_missing" || conditionType === "header_present") {
    if (!HEADER_NAME_RE.test(trimmedValue)) {
      return { error: "Header names can only contain RFC-compatible token characters." };
    }
  }

  if (conditionType === "header_missing") {
    return { condition: { header_missing: { header: trimmedValue } } };
  }

  if (conditionType === "header_present") {
    return { condition: { header_present: { header: trimmedValue } } };
  }

  return { condition: { body_contains: { value: trimmedValue } } };
}

function serializeRule(rule: CustomRule): string {
  return stringify(rule, {
    defaultStringType: "QUOTE_DOUBLE",
    lineWidth: 0,
  });
}

export function CreateRuleModal({ isOpen, onClose, onSubmit, isSubmitting }: CreateRuleModalProps) {
  const [name, setName] = useState("");
  const [category, setCategory] = useState<RuleCategory>("injection");
  const [severity, setSeverity] = useState<RuleSeverity>("high");
  const [confidence] = useState<RuleConfidence>("tentative");
  const [conditionType, setConditionType] = useState<ConditionType>("body_contains");
  const [conditionValue, setConditionValue] = useState("");
  const [formError, setFormError] = useState<string | null>(null);

  const finalId = buildRuleId(name);

  const buildRule = (requireValue: boolean): { rule: CustomRule } | { error: string } => {
    const conditionResult = createCondition(conditionType, conditionValue, requireValue);
    if ("error" in conditionResult) return conditionResult;

    return {
      rule: {
        id: finalId,
        name: name.trim() || "Unnamed Rule",
        description: "Custom user-defined rule",
        category,
        severity,
        confidence,
        source: "user-custom",
        pack: "custom",
        condition: conditionResult.condition,
      },
    };
  };

  const previewResult = buildRule(false);
  const previewError = "error" in previewResult ? previewResult.error : null;
  const yamlContent = "rule" in previewResult
    ? serializeRule(previewResult.rule)
    : `# ${previewResult.error}`;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setFormError(null);

    if (!name.trim()) {
      setFormError("Rule name is required.");
      return;
    }

    const result = buildRule(true);
    if ("error" in result) {
      setFormError(result.error);
      return;
    }

    onSubmit({ id: finalId, name: name.trim(), yaml_content: serializeRule(result.rule) });
  };

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <DialogContent
        showCloseButton={false}
        className="w-[95vw] max-w-3xl bg-[#0a0a0c] border-white/[0.1] p-0 shadow-2xl max-h-[90vh] overflow-hidden"
      >
        <DialogHeader className="px-6 py-4 border-b border-white/[0.06]">
          <div className="flex items-center justify-between gap-4">
            <DialogTitle className="text-lg font-bold text-foreground">Create Custom Rule</DialogTitle>
            <DialogClose asChild>
              <button
                type="button"
                className="p-1.5 rounded-md text-muted-foreground/60 hover:text-foreground hover:bg-white/[0.05] transition-colors"
                aria-label="Close create rule dialog"
              >
                <X className="w-5 h-5" />
              </button>
            </DialogClose>
          </div>
          <DialogDescription className="sr-only">
            Create a custom detection rule and preview the generated YAML.
          </DialogDescription>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto p-6 flex flex-col md:flex-row gap-6">
          <form id="create-rule-form" onSubmit={handleSubmit} className="flex-1 space-y-4">
            <div className="space-y-1.5">
              <label htmlFor="custom-rule-name" className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Rule Name</label>
              <input
                id="custom-rule-name"
                type="text"
                required
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g. Detect internal IP disclosure"
                className="w-full bg-black/40 border border-white/[0.1] rounded-md px-3 py-2 text-sm text-foreground focus:outline-none focus:border-primary/50 transition-colors"
              />
              <p className="text-[10px] text-muted-foreground/50 mt-1 font-mono">ID: {finalId}</p>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-1.5">
                <label htmlFor="custom-rule-category" className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Category</label>
                <select
                  id="custom-rule-category"
                  value={category}
                  onChange={(e) => setCategory(e.target.value as RuleCategory)}
                  className="w-full bg-black/40 border border-white/[0.1] rounded-md px-3 py-2 text-sm text-foreground focus:outline-none focus:border-primary/50"
                >
                  <option value="injection">Injection</option>
                  <option value="xss">XSS</option>
                  <option value="ssrf">SSRF</option>
                  <option value="authentication">Authentication</option>
                  <option value="misconfiguration">Misconfiguration</option>
                  <option value="information_disclosure">Information Disclosure</option>
                </select>
              </div>

              <div className="space-y-1.5">
                <label htmlFor="custom-rule-severity" className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Severity</label>
                <select
                  id="custom-rule-severity"
                  value={severity}
                  onChange={(e) => setSeverity(e.target.value as RuleSeverity)}
                  className="w-full bg-black/40 border border-white/[0.1] rounded-md px-3 py-2 text-sm text-foreground focus:outline-none focus:border-primary/50"
                >
                  <option value="critical">Critical</option>
                  <option value="high">High</option>
                  <option value="medium">Medium</option>
                  <option value="low">Low</option>
                  <option value="informational">Informational</option>
                </select>
              </div>
            </div>

            <div className="space-y-1.5 mt-4 pt-4 border-t border-white/[0.06]">
              <h3 className="text-[13px] font-bold text-foreground/90 mb-2">Detection Condition</h3>

              <div className="space-y-3">
                <select
                  aria-label="Condition type"
                  value={conditionType}
                  onChange={(e) => {
                    setConditionType(e.target.value as ConditionType);
                    setConditionValue("");
                    setFormError(null);
                  }}
                  className="w-full bg-black/40 border border-white/[0.1] rounded-md px-3 py-2 text-sm text-foreground focus:outline-none focus:border-primary/50"
                >
                  <option value="body_contains">Body Contains Text</option>
                  <option value="header_missing">Header is Missing</option>
                  <option value="header_present">Header is Present</option>
                  <option value="status_code_in">Status Code In List</option>
                </select>

                <input
                  aria-label="Condition value"
                  type="text"
                  required
                  value={conditionValue}
                  onChange={(e) => {
                    setConditionValue(e.target.value);
                    setFormError(null);
                  }}
                  placeholder={
                    conditionType === "body_contains" ? "e.g. root:x:0:0" :
                    conditionType === "status_code_in" ? "e.g. 500, 502, 503" :
                    "e.g. x-powered-by"
                  }
                  className={cn(
                    "w-full bg-black/40 border border-white/[0.1] rounded-md px-3 py-2 text-sm text-foreground focus:outline-none focus:border-primary/50 font-mono",
                    (previewError || formError) && "border-red-500/50 focus:border-red-500/70"
                  )}
                />
              </div>
            </div>

            {(previewError || formError) && (
              <div className="bg-red-500/10 border border-red-500/30 rounded-md p-3 flex items-start gap-2">
                <AlertCircle className="w-4 h-4 text-red-400 shrink-0 mt-0.5" />
                <p className="text-[11px] text-red-300/80 leading-relaxed">{formError ?? previewError}</p>
              </div>
            )}

            <div className="bg-primary/5 border border-primary/20 rounded-md p-3 flex items-start gap-2 mt-4">
              <AlertCircle className="w-4 h-4 text-primary shrink-0 mt-0.5" />
              <p className="text-[11px] text-muted-foreground/80 leading-relaxed">
                Custom rules are automatically loaded into the engine and will be executed on all subsequent scans.
              </p>
            </div>
          </form>

          <div className="flex-1 flex flex-col border border-white/[0.06] rounded-md overflow-hidden bg-[#050505] min-h-[320px]">
            <div className="bg-white/[0.02] px-3 py-2 border-b border-white/[0.06] flex items-center justify-between">
              <span className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/70">YAML Preview</span>
              <span className="text-[10px] font-mono text-muted-foreground/40">{finalId}.yaml</span>
            </div>
            <pre className="flex-1 p-4 text-[11px] font-mono text-primary/80 overflow-auto">
              {yamlContent}
            </pre>
          </div>
        </div>

        <div className="px-6 py-4 border-t border-white/[0.06] flex items-center justify-end gap-3 bg-black/20">
          <DialogClose asChild>
            <button
              type="button"
              className="px-4 py-2 text-sm font-medium text-foreground hover:bg-white/[0.05] rounded-md transition-colors"
            >
              Cancel
            </button>
          </DialogClose>
          <button
            type="submit"
            form="create-rule-form"
            disabled={isSubmitting || !name.trim() || !conditionValue.trim() || Boolean(previewError)}
            className="flex items-center gap-2 px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? <RefreshCw className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
            Save & Load Rule
          </button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
