import { toast } from "sonner";
import type { Finding } from "@/entities/finding/model/types";

export function exportFindingsAsCsv(findings: Finding[], filename = "limma_findings.csv") {
  if (!findings || findings.length === 0) {
    toast.error("No findings to export.");
    return;
  }

  try {
    const headers = ["ID", "Title", "Severity", "Confidence", "CVSS", "CWE", "Detector", "URL", "Parameter"];
    const rows = findings.map(f => [
      f.id,
      `"${f.title.replace(/"/g, '""')}"`,
      f.severity,
      f.confidence,
      f.cvss?.toString() || "",
      f.cwe || "",
      f.detector,
      `"${f.url.replace(/"/g, '""')}"`,
      f.parameter || ""
    ]);

    const csvContent = [headers.join(","), ...rows.map(r => r.join(","))].join("\n");
    
    downloadFile(csvContent, filename, "text/csv;charset=utf-8;");
    toast.success("Findings exported successfully as CSV.");
  } catch (err) {
    toast.error("Failed to export findings.");
    console.error(err);
  }
}

export function exportFindingsAsJson(findings: Finding[], filename = "limma_findings.json") {
  if (!findings || findings.length === 0) {
    toast.error("No findings to export.");
    return;
  }

  try {
    const jsonContent = JSON.stringify(findings, null, 2);
    downloadFile(jsonContent, filename, "application/json;charset=utf-8;");
    toast.success("Findings exported successfully as JSON.");
  } catch (err) {
    toast.error("Failed to export findings.");
    console.error(err);
  }
}

function downloadFile(content: string, filename: string, type: string) {
  const blob = new Blob([content], { type });
  const link = document.createElement("a");
  
  if (link.download !== undefined) {
    const url = URL.createObjectURL(blob);
    link.setAttribute("href", url);
    link.setAttribute("download", filename);
    link.style.visibility = "hidden";
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }
}
