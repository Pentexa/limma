/** All application route paths */
export const ROUTES = {
  DASHBOARD: "/",
  SCANNER: "/scanner",
  AUDIT: "/audit",
  ACTIVE_DETECTION: "/active-detection",
  POC_LAB: "/poc-lab",
  REPORTS: "/reports",
  SETTINGS: "/settings",
  DISCOVERY: "/discovery",
  ANALYSIS: "/analysis",
  FINDING_DETAIL: "/findings",
  HISTORY: "/history",
  INTEGRATIONS: "/integrations",
  RULES: "/rules",
  BLIND_SCANNER: "/blind-scanner",
  HTTP_REQUESTER: "/http-requester",
} as const;

export type RoutePath = (typeof ROUTES)[keyof typeof ROUTES];
