import { httpClient } from "@/shared/api/http-client";
import type { ApiSettingsProfile } from "@/shared/types/api";

/** Fetch all settings profiles */
export async function fetchSettingsProfiles(): Promise<ApiSettingsProfile[]> {
  return httpClient.get<ApiSettingsProfile[]>("/api/settings/profiles");
}

/** Update a settings profile */
export async function updateSettingsProfile(
  id: string,
  data: Partial<ApiSettingsProfile>
): Promise<ApiSettingsProfile> {
  return httpClient.put<ApiSettingsProfile>(`/api/settings/profiles/${id}`, data);
}
