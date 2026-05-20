"use client";

import { useMutation } from "@tanstack/react-query";
import {
  analyzeWebsite,
  investigateServer,
  discoverApis,
  collectServices,
  auditSecurity,
  mapForms,
  type IntelRequest,
} from "../api/intelligence-api";

export function useAnalyzeWebsite() {
  return useMutation<Record<string, unknown>, Error, IntelRequest>({
    mutationFn: analyzeWebsite,
  });
}

export function useInvestigateServer() {
  return useMutation<Record<string, unknown>, Error, IntelRequest>({
    mutationFn: investigateServer,
  });
}

export function useDiscoverApis() {
  return useMutation<Record<string, unknown>, Error, IntelRequest>({
    mutationFn: discoverApis,
  });
}

export function useCollectServices() {
  return useMutation<Record<string, unknown>, Error, IntelRequest>({
    mutationFn: collectServices,
  });
}

export function useAuditSecurity() {
  return useMutation<Record<string, unknown>, Error, IntelRequest>({
    mutationFn: auditSecurity,
  });
}

export function useMapForms() {
  return useMutation<Record<string, unknown>, Error, IntelRequest>({
    mutationFn: mapForms,
  });
}
