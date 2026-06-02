import { Search, X } from "lucide-react";
import { useState } from "react";
import { cn } from "@/shared/lib/utils";

interface FindingFilterProps {
  onFilterChange: (filters: { query: string }) => void;
  className?: string;
}

export function FindingFilter({ onFilterChange, className }: FindingFilterProps) {
  const [query, setQuery] = useState("");

  const handleQueryChange = (val: string) => {
    setQuery(val);
    onFilterChange({ query: val });
  };

  const clearQuery = () => {
    setQuery("");
    onFilterChange({ query: "" });
  };

  return (
    <div className={cn("flex items-center gap-2", className)}>
      <div className="relative flex-1 min-w-[200px]">
        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground" />
        <input
          type="text"
          value={query}
          onChange={(e) => handleQueryChange(e.target.value)}
          placeholder="Filter by URL, parameter, or title..."
          className="w-full pl-8 pr-8 h-8 bg-muted/30 border border-border rounded text-[11px] font-mono text-foreground placeholder:text-muted-foreground/60 focus:outline-none focus:ring-1 focus:ring-primary/40 focus:border-primary/40 transition-colors"
        />
        {query && (
          <button 
            onClick={clearQuery}
            className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
          >
            <X className="h-3.5 w-3.5" />
          </button>
        )}
      </div>
    </div>
  );
}
