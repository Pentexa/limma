"use client";

import { Search, Compass, MapPin, Globe, Cloud, Key, FileWarning, MoveRight, ArrowRight } from "lucide-react";

export function DiscoveryScreen() {
  const futureModules = [
    { name: "Subdomain Discovery", description: "Automatically enumerate and validate subdomains.", icon: Globe },
    { name: "Cloud Asset Discovery", description: "Map out AWS, GCP, and Azure storage buckets and instances.", icon: Cloud },
    { name: "Leak Discovery", description: "Search for exposed source code and sensitive documents.", icon: Key },
    { name: "Certificate Discovery", description: "Scan CT logs to discover related domains.", icon: FileWarning },
    { name: "External Exposure Map", description: "Visualize full internet-facing attack surface.", icon: MapPin },
    { name: "Attack Surface Expansion", description: "Recursive analysis to find adjacent targets.", icon: Compass },
  ];

  const movedModules = [
    "API Discovery",
    "Service Collection",
    "Server Information",
    "Technology Stack",
  ];

  return (
    <div className="flex flex-col h-full w-full max-w-full min-w-0 overflow-hidden bg-[#0a0a0c]">
      {/* Top bar */}
      <div className="shrink-0 flex items-center justify-between gap-4 px-5 py-3 border-b border-white/[0.06]">
        <div className="flex items-center gap-3">
          <div className="flex items-center justify-center h-7 w-7 rounded-lg bg-primary/10 border border-primary/20">
            <Compass className="h-3.5 w-3.5 text-primary" />
          </div>
          <div>
            <h2 className="text-sm font-semibold text-foreground">Discovery Roadmap</h2>
            <p className="text-[11px] font-mono text-muted-foreground/60">
              Future intelligence gathering modules
            </p>
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-8">
        <div className="max-w-4xl mx-auto space-y-12 animate-in fade-in slide-in-from-bottom-4 duration-700">
          
          {/* Header */}
          <div className="text-center space-y-3">
            <h1 className="text-2xl font-bold text-foreground tracking-tight">The Future of Discovery</h1>
            <p className="text-muted-foreground/70 max-w-xl mx-auto text-sm">
              Active discovery modules have been consolidated into the unified Scanner dashboard. 
              This space will host our upcoming passive footprinting and deep-dive discovery engines.
            </p>
          </div>

          {/* Moved Note */}
          <div className="bg-primary/[0.02] border border-primary/20 rounded-xl p-6 shadow-inner relative overflow-hidden">
            <div className="absolute top-0 left-0 w-1 h-full bg-primary/50" />
            <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
              <div className="space-y-2">
                <h3 className="text-[14px] font-bold text-primary flex items-center gap-2">
                  <MoveRight className="h-4 w-4" /> Consolidated Features
                </h3>
                <p className="text-[12px] text-muted-foreground/80">
                  The following active discovery modules have been moved to the <strong className="text-foreground">Scanner</strong> module for a more unified workflow:
                </p>
                <div className="flex flex-wrap gap-2 pt-2">
                  {movedModules.map(mod => (
                    <span key={mod} className="text-[11px] font-mono bg-black/40 border border-white/[0.05] px-2.5 py-1 rounded-md text-muted-foreground">
                      {mod}
                    </span>
                  ))}
                </div>
              </div>
              <button className="shrink-0 flex items-center gap-2 bg-primary/10 hover:bg-primary/20 border border-primary/20 text-primary px-4 py-2 rounded-lg text-[12px] font-semibold transition-colors">
                Go to Scanner <ArrowRight className="h-3 w-3" />
              </button>
            </div>
          </div>

          {/* Roadmap Grid */}
          <div className="space-y-4">
            <h3 className="text-[13px] font-bold uppercase tracking-wider text-foreground/80 flex items-center gap-2">
              <span className="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse" />
              Planned Modules
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {futureModules.map((mod, i) => (
                <div key={i} className="bg-white/[0.01] border border-white/[0.04] p-5 rounded-xl hover:border-white/[0.08] transition-colors group">
                  <div className="flex items-center gap-4">
                    <div className="h-10 w-10 rounded-lg bg-black/40 border border-white/[0.03] flex items-center justify-center group-hover:scale-110 transition-transform">
                      <mod.icon className="h-5 w-5 text-muted-foreground/60 group-hover:text-primary transition-colors" />
                    </div>
                    <div>
                      <h4 className="text-[14px] font-bold text-foreground/90">{mod.name}</h4>
                      <p className="text-[12px] text-muted-foreground/60 mt-0.5">{mod.description}</p>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>

        </div>
      </div>
    </div>
  );
}
