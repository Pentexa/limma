import { httpClient } from "@/shared/api/http-client";
import type { Rule } from "../model/types";

export async function fetchRules(): Promise<Rule[]> {
  return httpClient.get<Rule[]>("/api/rules");
}

export async function fetchRule(id: string): Promise<Rule> {
  return httpClient.get<Rule>(`/api/rules/${id}`);
}
