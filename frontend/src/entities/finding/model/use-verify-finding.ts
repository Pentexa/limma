"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { verifyFinding, updateFindingStatus } from "@/features/verify-finding/api/verify-finding";
import { findingKeys } from "./use-findings";
import type { ApiExploitResult } from "@/shared/types/api";
import type { FindingId } from "@/shared/types/common";
import type { Finding } from "./types";
import { toast } from "sonner";

/** Trigger automated exploit-based verification for a finding (PoC Lab) */
export function useAutoExploitFinding() {
  const queryClient = useQueryClient();

  return useMutation<ApiExploitResult, Error, FindingId>({
    mutationFn: (findingId: FindingId) => verifyFinding(findingId),
    onSuccess: (data, findingId) => {
      // If exploit succeeds, optionally mark it verified locally
      if (data.success) {
        queryClient.setQueryData<Finding | undefined>(findingKeys.detail(findingId), (old: Finding | undefined) => {
          if (!old) return old;
          return { ...old, verification: "verified" };
        });
      }
      queryClient.invalidateQueries({ queryKey: findingKeys.all });
      toast.success(data.success ? "Exploit successful!" : "Exploit failed to verify.");
    },
    onError: (err) => {
      toast.error(err instanceof Error ? err.message : "Failed to run automated exploit");
    }
  });
}

/** Manual verification (Active Detection) */
export function useVerifyFinding() {
  const queryClient = useQueryClient();

  return useMutation<void, Error, FindingId>({
    mutationFn: (findingId: FindingId) => updateFindingStatus(findingId, { verified: true, false_positive: false }),
    onSuccess: (_, findingId) => {
      // Optimistic/immediate local cache update for detail view
      queryClient.setQueryData<Finding | undefined>(findingKeys.detail(findingId), (old: Finding | undefined) => {
        if (!old) return old;
        return { ...old, verification: "verified" };
      });
      // Also update lists if possible or let invalidate handle it
      queryClient.invalidateQueries({ queryKey: findingKeys.all });
      toast.success("Finding verified successfully");
    },
    onError: (err) => {
      toast.error(err instanceof Error ? err.message : "Failed to verify finding");
    }
  });
}

/** Mark a finding as verified or false positive */
export function useUpdateFindingStatus() {
  const queryClient = useQueryClient();

  return useMutation<void, Error, { findingId: FindingId; verified: boolean; falsePositive: boolean }>({
    mutationFn: ({ findingId, verified, falsePositive }) =>
      updateFindingStatus(findingId, { verified, false_positive: falsePositive }),
    onSuccess: (_, variables) => {
      const newStatus = variables.verified ? "verified" : variables.falsePositive ? "false_positive" : "unverified";
      
      queryClient.setQueryData<Finding | undefined>(findingKeys.detail(variables.findingId), (old: Finding | undefined) => {
        if (!old) return old;
        return { ...old, verification: newStatus };
      });

      queryClient.invalidateQueries({ queryKey: findingKeys.all });
      toast.success(variables.falsePositive ? "Marked as false positive" : "Status updated");
    },
    onError: (err) => {
      toast.error(err instanceof Error ? err.message : "Failed to update status");
    }
  });
}
