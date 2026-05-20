"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { burpListSessions, burpHandshake, burpGetFindings } from "../api/connect-burp";
import type { ApiBurpSession, ApiBurpFinding } from "@/shared/types/api";

export const burpKeys = {
  all: ["burp"] as const,
  sessions: () => [...burpKeys.all, "sessions"] as const,
  findings: (sessionId: string) => [...burpKeys.all, "findings", sessionId] as const,
};

/** Fetch all Burp sessions */
export function useBurpSessions() {
  return useQuery<ApiBurpSession[], Error>({
    queryKey: burpKeys.sessions(),
    queryFn: burpListSessions,
    refetchInterval: 30000,
  });
}

/** Fetch findings for a specific Burp session */
export function useBurpFindings(sessionId: string | undefined) {
  return useQuery<ApiBurpFinding[], Error>({
    queryKey: burpKeys.findings(sessionId ?? ""),
    queryFn: () => burpGetFindings(sessionId!),
    enabled: !!sessionId,
  });
}

/** Initiate Burp handshake */
export function useBurpHandshake() {
  const queryClient = useQueryClient();
  return useMutation<{ session_id: string }, Error, string>({
    mutationFn: (pluginVersion: string) => burpHandshake(pluginVersion),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: burpKeys.sessions() });
    },
  });
}
