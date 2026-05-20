"use client";

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/shared/ui/select";
import { Download } from "lucide-react";
import { useState } from "react";
import { exportToBurp } from "../api/export-to-burp";
import { exportToNuclei } from "../api/export-to-nuclei";
import { toast } from "sonner";

interface ExportDropdownProps { scanId: string; disabled?: boolean; }

export function ExportDropdown({ scanId, disabled }: ExportDropdownProps) {
  const [isExporting, setIsExporting] = useState(false);

  async function handleExport(format: string) {
    setIsExporting(true);
    try {
      if (format === "burp") await exportToBurp(scanId);
      else if (format === "nuclei") await exportToNuclei(scanId);
    } catch (err) { toast.error(err instanceof Error ? err.message : "Export failed"); } finally { setIsExporting(false); }
  }

  return (
    <Select onValueChange={handleExport} disabled={disabled || isExporting}>
      <SelectTrigger className="w-[140px] h-8 text-xs">
        <div className="flex items-center gap-1.5">
          <Download className="h-3.5 w-3.5" />
          <SelectValue placeholder="Export" />
        </div>
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="burp">Burp Suite</SelectItem>
        <SelectItem value="nuclei">Nuclei</SelectItem>
      </SelectContent>
    </Select>
  );
}
