import type { ScanSession } from './scanSessionStore';
import type { MasterReport } from './api';

export interface DeltaAnalysisResult {
  hasChanges: boolean;
  scoreDelta: number;
  newFindings: Array<{ slug: string; priority: string; title: string }>;
  resolvedFindings: Array<{ slug: string; priority: string; title: string }>;
  newPorts: number[];
  closedPorts: number[];
  newApis: string[];
  removedApis: string[];
}

export function calculateDelta(current: ScanSession, previous: ScanSession): DeltaAnalysisResult {
  const currentMaster = current.moduleResults['dashboard']?.result as MasterReport | undefined;
  const previousMaster = previous.moduleResults['dashboard']?.result as MasterReport | undefined;

  const result: DeltaAnalysisResult = {
    hasChanges: false,
    scoreDelta: 0,
    newFindings: [],
    resolvedFindings: [],
    newPorts: [],
    closedPorts: [],
    newApis: [],
    removedApis: [],
  };

  if (!currentMaster || !previousMaster) {
    return result; // Not enough data to compare
  }

  // 1. Score Delta
  result.scoreDelta = currentMaster.overall_health_score - previousMaster.overall_health_score;

  // 2. Findings (Canonical Slugs)
  const currentFindingsMap = new Map(
    (currentMaster.normalized_audit?.canonical_findings || []).map((f) => [f.canonical_slug, f])
  );
  const previousFindingsMap = new Map(
    (previousMaster.normalized_audit?.canonical_findings || []).map((f) => [f.canonical_slug, f])
  );

  // New Findings (in current, not in previous)
  for (const [slug, finding] of currentFindingsMap.entries()) {
    if (!previousFindingsMap.has(slug)) {
      result.newFindings.push({ slug, priority: finding.severity, title: finding.title });
    }
  }

  // Resolved Findings (in previous, not in current)
  for (const [slug, finding] of previousFindingsMap.entries()) {
    if (!currentFindingsMap.has(slug)) {
      result.resolvedFindings.push({ slug, priority: finding.severity, title: finding.title });
    }
  }

  // 3. Ports
  const currentPorts = new Set(
    (currentMaster.service_collector?.port_results || [])
      .filter((p) => p.state.toLowerCase() === 'open')
      .map((p) => p.port)
  );
  const previousPorts = new Set(
    (previousMaster.service_collector?.port_results || [])
      .filter((p) => p.state.toLowerCase() === 'open')
      .map((p) => p.port)
  );

  currentPorts.forEach((port) => {
    if (!previousPorts.has(port)) result.newPorts.push(port);
  });
  previousPorts.forEach((port) => {
    if (!currentPorts.has(port)) result.closedPorts.push(port);
  });

  // 4. APIs
  const currentApis = new Set(
    (currentMaster.api_discovery?.detected_endpoints || []).map((e) => `${e.method_prediction} ${e.path}`)
  );
  const previousApis = new Set(
    (previousMaster.api_discovery?.detected_endpoints || []).map((e) => `${e.method_prediction} ${e.path}`)
  );

  currentApis.forEach((api) => {
    if (!previousApis.has(api)) result.newApis.push(api);
  });
  previousApis.forEach((api) => {
    if (!currentApis.has(api)) result.removedApis.push(api);
  });

  // Determine if there are any changes
  result.hasChanges =
    result.scoreDelta !== 0 ||
    result.newFindings.length > 0 ||
    result.resolvedFindings.length > 0 ||
    result.newPorts.length > 0 ||
    result.closedPorts.length > 0 ||
    result.newApis.length > 0 ||
    result.removedApis.length > 0;

  return result;
}
