import { httpClient } from "@/shared/api/http-client";

export interface IntelRequest {
  url: string;
  profile_id?: string;
}

/** POST /analyze — Website technology + header + risk analysis */
export function analyzeWebsite(data: IntelRequest) {
  return httpClient.post<Record<string, unknown>>("/analyze", data);
}

/** POST /investigate — Server fingerprinting + DNS investigation */
export function investigateServer(data: IntelRequest) {
  return httpClient.post<Record<string, unknown>>("/investigate", data);
}

/** POST /discover-apis — API endpoint discovery */
export function discoverApis(data: IntelRequest) {
  return httpClient.post<Record<string, unknown>>("/discover-apis", data);
}

/** POST /collect-services — Port/service collection */
export function collectServices(data: IntelRequest) {
  return httpClient.post<Record<string, unknown>>("/collect-services", data);
}

/** POST /audit-security — Security header + vulnerability audit */
export function auditSecurity(data: IntelRequest) {
  return httpClient.post<Record<string, unknown>>("/audit-security", data);
}

/** POST /map-forms — HTML form mapping */
export function mapForms(data: IntelRequest) {
  return httpClient.post<Record<string, unknown>>("/map-forms", data);
}
