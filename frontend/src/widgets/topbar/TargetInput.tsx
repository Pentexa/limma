"use client";

import { cn } from "@/shared/lib/utils";
import { Search, ArrowRight } from "lucide-react";

import { forwardRef } from "react";

interface TargetInputProps extends Omit<React.InputHTMLAttributes<HTMLInputElement>, 'onSubmit'> {
  onTargetSubmit?: (value: string) => void;
}

export const TargetInput = forwardRef<HTMLInputElement, TargetInputProps>(
  ({ value, onChange, onTargetSubmit, className, ...props }, ref) => {
    return (
    <div className={cn("relative flex items-center", className)}>
      <Search className="absolute left-2.5 h-3.5 w-3.5 text-muted-foreground" />
        <input
          ref={ref}
          value={value}
          onChange={onChange}
          onKeyDown={(e) => e.key === "Enter" && value && onTargetSubmit?.(value as string)}
          placeholder="Target URL or domain…"
          aria-label="Target URL input"
          className="w-full pl-8 pr-8 h-8 bg-white/[0.04] border border-white/[0.06] rounded-md text-[12px] font-mono text-foreground placeholder:text-muted-foreground/60 focus:outline-none focus:ring-1 focus:ring-primary/40 focus:border-primary/40 transition-all duration-200"
          {...props}
        />
        {value && onTargetSubmit && (
          <button type="button" aria-label="Submit target URL" className="absolute right-2 text-muted-foreground hover:text-primary" onClick={() => onTargetSubmit?.(value as string)}>
            <ArrowRight className="h-3.5 w-3.5" />
          </button>
        )}
      </div>
    );
  }
);

TargetInput.displayName = "TargetInput";
