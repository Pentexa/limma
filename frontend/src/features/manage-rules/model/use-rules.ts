"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { fetchRuleEngineStatus, fetchFeedbackStats, submitRuleFeedback, createCustomRule, deleteRule } from "../api/rule-actions";
import type { ApiRuleEngineStatus, ApiFeedbackStatsResponse } from "@/shared/types/api";

export const ruleKeys = {
  all: ["rules"] as const,
  engineStatus: () => [...ruleKeys.all, "engine-status"] as const,
  feedbackStats: () => [...ruleKeys.all, "feedback-stats"] as const,
};

/** Fetch rule engine status */
export function useRuleEngineStatus() {
  return useQuery<ApiRuleEngineStatus, Error>({
    queryKey: ruleKeys.engineStatus(),
    queryFn: fetchRuleEngineStatus,
    refetchInterval: 30000,
  });
}

/** Fetch feedback statistics */
export function useFeedbackStats() {
  return useQuery<ApiFeedbackStatsResponse, Error>({
    queryKey: ruleKeys.feedbackStats(),
    queryFn: fetchFeedbackStats,
    refetchInterval: 60000,
  });
}

/** Submit feedback for a rule */
export function useSubmitFeedback() {
  const queryClient = useQueryClient();
  return useMutation<void, Error, { ruleId: string; targetUrl: string; action: string }>({
    mutationFn: ({ ruleId, targetUrl, action }) => submitRuleFeedback(ruleId, targetUrl, action),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ruleKeys.feedbackStats() });
    },
  });
}

/** Create a custom rule */
export function useCreateRule() {
  const queryClient = useQueryClient();
  return useMutation<void, Error, { id: string; name: string; yaml_content: string }>({
    mutationFn: createCustomRule,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ruleKeys.engineStatus() });
    },
  });
}

/** Delete a custom rule */
export function useDeleteRule() {
  const queryClient = useQueryClient();
  return useMutation<void, Error, string>({
    mutationFn: deleteRule,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ruleKeys.engineStatus() });
    },
  });
}
