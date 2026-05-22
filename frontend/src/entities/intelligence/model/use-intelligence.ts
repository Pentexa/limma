"use client";

import { useMutation } from "@tanstack/react-query";
import {
  analyzeWebsite,
  investigateServer,
  discoverApis,
  collectServices,
  auditSecurity,
  mapForms,
  generateMasterReport,
  type IntelRequest,
} from "../api/intelligence-api";
import type { 
  ApiWebScanResult, 
  ApiServerInfo, 
  ApiDiscoveryResult, 
  ApiCollectorSnapshot, 
  ApiSecurityReport, 
  ApiFormMapping,
  ApiMasterReport
} from "@/shared/types/api";

export function useAnalyzeWebsite() {
  return useMutation<ApiWebScanResult, Error, IntelRequest>({
    mutationFn: analyzeWebsite,
  });
}

export function useInvestigateServer() {
  return useMutation<ApiServerInfo, Error, IntelRequest>({
    mutationFn: investigateServer,
  });
}

export function useDiscoverApis() {
  return useMutation<ApiDiscoveryResult, Error, IntelRequest>({
    mutationFn: discoverApis,
  });
}

export function useCollectServices() {
  return useMutation<ApiCollectorSnapshot, Error, IntelRequest>({
    mutationFn: collectServices,
  });
}

export function useAuditSecurity() {
  return useMutation<ApiSecurityReport, Error, IntelRequest>({
    mutationFn: auditSecurity,
  });
}

export function useMapForms() {
  return useMutation<ApiFormMapping, Error, IntelRequest>({
    mutationFn: mapForms,
  });
}

export function useGenerateMasterReport() {
  return useMutation<ApiMasterReport, Error, IntelRequest>({
    mutationFn: generateMasterReport,
  });
}
