import React, { useState } from "react";
import { X, Save, AlertCircle, RefreshCw } from "lucide-react";

interface CreateRuleModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (payload: { id: string; name: string; yaml_content: string }) => void;
  isSubmitting: boolean;
}

export function CreateRuleModal({ isOpen, onClose, onSubmit, isSubmitting }: CreateRuleModalProps) {
  const [name, setName] = useState("");
  const [category, setCategory] = useState("injection");
  const [severity, setSeverity] = useState("high");
  const [confidence] = useState("tentative");
  const [conditionType, setConditionType] = useState("body_contains");
  const [conditionValue, setConditionValue] = useState("");
  
  if (!isOpen) return null;

  // Auto-generate ID from name
  const ruleId = name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/(^-|-$)+/g, "");
    
  const finalId = ruleId ? `custom-${ruleId}` : "custom-rule-id";

  const generateYaml = () => {
    let conditionYaml = "";
    if (conditionType === "body_contains") {
      conditionYaml = `  body_contains:\n    value: "${conditionValue}"`;
    } else if (conditionType === "header_missing") {
      conditionYaml = `  header_missing:\n    header: "${conditionValue}"`;
    } else if (conditionType === "header_present") {
      conditionYaml = `  header_present:\n    header: "${conditionValue}"`;
    } else if (conditionType === "status_code_in") {
      conditionYaml = `  status_code_in:\n    codes: [${conditionValue}]`;
    }

    return `id: "${finalId}"
name: "${name || "Unnamed Rule"}"
description: "Custom user-defined rule"
category: "${category}"
severity: "${severity}"
confidence: "${confidence}"
source: "user-custom"
pack: "custom"
condition:
${conditionYaml}
`;
  };

  const yamlContent = generateYaml();

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name || !conditionValue) return;
    onSubmit({ id: finalId, name, yaml_content: yamlContent });
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
      <div className="w-full max-w-3xl bg-[#0a0a0c] border border-white/[0.1] rounded-xl shadow-2xl flex flex-col max-h-[90vh]">
        
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-white/[0.06]">
          <h2 className="text-lg font-bold text-foreground">Create Custom Rule</h2>
          <button 
            onClick={onClose}
            className="p-1.5 rounded-md text-muted-foreground/60 hover:text-foreground hover:bg-white/[0.05] transition-colors"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 flex flex-col md:flex-row gap-6">
          
          {/* Form */}
          <form id="create-rule-form" onSubmit={handleSubmit} className="flex-1 space-y-4">
            
            <div className="space-y-1.5">
              <label className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Rule Name</label>
              <input 
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
                <label className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Category</label>
                <select 
                  value={category}
                  onChange={(e) => setCategory(e.target.value)}
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
                <label className="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider">Severity</label>
                <select 
                  value={severity}
                  onChange={(e) => setSeverity(e.target.value)}
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
                  value={conditionType}
                  onChange={(e) => {
                    setConditionType(e.target.value);
                    setConditionValue("");
                  }}
                  className="w-full bg-black/40 border border-white/[0.1] rounded-md px-3 py-2 text-sm text-foreground focus:outline-none focus:border-primary/50"
                >
                  <option value="body_contains">Body Contains Text</option>
                  <option value="header_missing">Header is Missing</option>
                  <option value="header_present">Header is Present</option>
                  <option value="status_code_in">Status Code In List</option>
                </select>
                
                <input 
                  type="text" 
                  required
                  value={conditionValue}
                  onChange={(e) => setConditionValue(e.target.value)}
                  placeholder={
                    conditionType === "body_contains" ? "e.g. root:x:0:0" :
                    conditionType === "status_code_in" ? "e.g. 500, 502, 503" :
                    "e.g. x-powered-by"
                  }
                  className="w-full bg-black/40 border border-white/[0.1] rounded-md px-3 py-2 text-sm text-foreground focus:outline-none focus:border-primary/50 font-mono"
                />
              </div>
            </div>
            
            <div className="bg-primary/5 border border-primary/20 rounded-md p-3 flex items-start gap-2 mt-4">
              <AlertCircle className="w-4 h-4 text-primary shrink-0 mt-0.5" />
              <p className="text-[11px] text-muted-foreground/80 leading-relaxed">
                Custom rules are automatically loaded into the engine and will be executed on all subsequent scans.
              </p>
            </div>
            
          </form>

          {/* YAML Preview */}
          <div className="flex-1 flex flex-col border border-white/[0.06] rounded-md overflow-hidden bg-[#050505]">
            <div className="bg-white/[0.02] px-3 py-2 border-b border-white/[0.06] flex items-center justify-between">
              <span className="text-[10px] font-bold uppercase tracking-wider text-muted-foreground/70">YAML Preview</span>
              <span className="text-[10px] font-mono text-muted-foreground/40">{finalId}.yaml</span>
            </div>
            <pre className="flex-1 p-4 text-[11px] font-mono text-primary/80 overflow-auto">
              {yamlContent}
            </pre>
          </div>
          
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-white/[0.06] flex items-center justify-end gap-3 bg-black/20">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-sm font-medium text-foreground hover:bg-white/[0.05] rounded-md transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            form="create-rule-form"
            disabled={isSubmitting || !name || !conditionValue}
            className="flex items-center gap-2 px-4 py-2 bg-primary hover:bg-primary/90 text-primary-foreground text-sm font-medium rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {isSubmitting ? <RefreshCw className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
            Save & Load Rule
          </button>
        </div>

      </div>
    </div>
  );
}
