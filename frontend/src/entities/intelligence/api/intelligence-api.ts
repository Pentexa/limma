import { httpClient } from "@/shared/api/http-client";
import type { 
  ApiWebScanResult, 
  ApiServerInfo, 
  ApiDiscoveryResult, 
  ApiCollectorSnapshot, 
  ApiSecurityReport, 
  ApiFormMapping,
  ApiMasterReport,
  ApiSubdomainDiscoveryResult,
  ApiSubdomainDiscoveryRequest,
  ApiDiscoverCertificatesRequest,
  ApiDiscoverCertificatesResponse,
} from "@/shared/types/api";

export interface IntelRequest {
  url: string;
  profile_id?: string;
}

/** POST /analyze — Website technology + header + risk analysis */
export function analyzeWebsite(data: IntelRequest) {
  return httpClient.post<ApiWebScanResult>("/analyze", data);
}

/** POST /investigate — Server fingerprinting + DNS investigation */
export function investigateServer(data: IntelRequest) {
  return httpClient.post<ApiServerInfo>("/investigate", data);
}

/** POST /discover-apis — API endpoint discovery */
export function discoverApis(data: IntelRequest) {
  return httpClient.post<ApiDiscoveryResult>("/discover-apis", data);
}

/** POST /collect-services — Port/service collection */
export function collectServices(data: IntelRequest) {
  return httpClient.post<ApiCollectorSnapshot>("/collect-services", data);
}

/** POST /audit-security — Security header + vulnerability audit */
export function auditSecurity(data: IntelRequest) {
  return httpClient.post<ApiSecurityReport>("/audit-security", data);
}

/** POST /map-forms — HTML form mapping */
export function mapForms(data: IntelRequest) {
  return httpClient.post<ApiFormMapping>("/map-forms", data);
}

/** POST /master-report — Full reconnaissance and correlation */
export function generateMasterReport(data: IntelRequest) {
  return httpClient.post<ApiMasterReport>("/master-report", data);
}

/** POST /discover-subdomains — Subdomain enumeration & validation */
export function discoverSubdomains(data: ApiSubdomainDiscoveryRequest) {
  return httpClient.post<ApiSubdomainDiscoveryResult>("/discover-subdomains", data);
}

/** POST /api/discovery/certificates — Certificate enumeration & validation */
export function discoverCertificates(data: ApiDiscoverCertificatesRequest) {
  return httpClient.post<ApiDiscoverCertificatesResponse>("/api/discovery/certificates", data);
}
