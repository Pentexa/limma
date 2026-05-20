import { httpClient } from "@/shared/api/http-client";
import type { ApiRuleEngineStatus, ApiFeedbackStatsResponse } from "@/shared/types/api";

/** Fetch rule engine status */
export async function fetchRuleEngineStatus(): Promise<ApiRuleEngineStatus> {
  return httpClient.get<ApiRuleEngineStatus>("/api/rule-engine-status");
}

/** Submit feedback for a dynamic rule */
export async function submitRuleFeedback(
  ruleId: string,
  targetUrl: string,
  action: string
): Promise<void> {
  return httpClient.post("/api/dynamic-rule/feedback", {
    rule_id: ruleId,
    target_url: targetUrl,
    action,
  });
}

/** Fetch feedback statistics */
export async function fetchFeedbackStats(): Promise<ApiFeedbackStatsResponse> {
  return httpClient.get<ApiFeedbackStatsResponse>("/api/feedback-stats");
}

/** Create a custom rule */
export async function createCustomRule(payload: { id: string; name: string; yaml_content: string }): Promise<void> {
  return httpClient.post("/api/rules", payload);
}

/** Delete a custom rule */
export async function deleteRule(ruleId: string): Promise<void> {
  return httpClient.delete(`/api/rules/${ruleId}`);
}
