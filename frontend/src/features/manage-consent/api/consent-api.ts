import { httpClient } from "@/shared/api/http-client";

export interface ConsentRecord {
  id: string;
  target_domain: string;
  consent_level: string;
  granted_by: string;
  granted_at: string;
  expires_at: string | null;
  revoked: boolean;
  revoked_at: string | null;
}

export interface GrantConsentRequest {
  target_domain: string;
  requested_by: string;
  scope_level: string;
  expires_in_hours: number;
}

export interface RevokeConsentRequest {
  target_domain: string;
}

export interface GrantConsentResponse {
  status: string;
  consent_id: string;
}

export interface RevokeConsentResponse {
  status: string;
}

/** GET /api/settings/consent - Fetch all active and past consents */
export function getConsents() {
  return httpClient.get<ConsentRecord[]>("/api/settings/consent");
}

/** POST /api/settings/consent - Grant new consent */
export function grantConsent(data: GrantConsentRequest) {
  return httpClient.post<GrantConsentResponse>("/api/settings/consent", data);
}

/** DELETE /api/settings/consent/:id - Revoke consent */
export function revokeConsent(id: string, target_domain: string) {
  // Use data payload in delete if backend expects json body, or query param
  // According to handlers.rs, it expects JSON payload
  return httpClient.delete<RevokeConsentResponse>(`/api/settings/consent/${id}`, { target_domain });
}
