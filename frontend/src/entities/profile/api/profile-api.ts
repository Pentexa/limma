import { httpClient } from "@/shared/api/http-client";
import type { ScanProfile } from "../model/types";

export async function fetchProfiles(): Promise<ScanProfile[]> {
  return httpClient.get<ScanProfile[]>("/api/profiles");
}

export async function fetchProfile(id: string): Promise<ScanProfile> {
  return httpClient.get<ScanProfile>(`/api/profiles/${id}`);
}
