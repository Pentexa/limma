"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { httpClient } from "@/shared/api/http-client";
import type { ApiSettingsProfile } from "@/shared/types/api";

export const settingsKeys = {
  all: ["settings"] as const,
  profiles: () => [...settingsKeys.all, "profiles"] as const,
  scanProfiles: () => [...settingsKeys.all, "scan-profiles"] as const,
};

/** Fetch settings profiles */
export function useSettingsProfiles() {
  return useQuery<ApiSettingsProfile[], Error>({
    queryKey: settingsKeys.profiles(),
    queryFn: () => httpClient.get<ApiSettingsProfile[]>("/api/settings/profiles"),
  });
}

/** Fetch scan profiles */
export function useScanProfiles() {
  return useQuery<ApiSettingsProfile[], Error>({
    queryKey: settingsKeys.scanProfiles(),
    queryFn: () => httpClient.get<ApiSettingsProfile[]>("/api/profiles"),
  });
}

/** Update a settings profile */
export function useUpdateProfile() {
  const queryClient = useQueryClient();
  return useMutation<ApiSettingsProfile, Error, { id: string; data: Partial<ApiSettingsProfile> }>({
    mutationFn: ({ id, data }) => httpClient.put<ApiSettingsProfile>(`/api/settings/profiles/${id}`, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.profiles() });
    },
  });
}
