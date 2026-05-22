import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { getConsents, grantConsent, revokeConsent } from "../api/consent-api";
import type { GrantConsentRequest } from "../api/consent-api";
import { toast } from "sonner";

export const consentKeys = {
  all: ["consents"] as const,
};

export function useConsents() {
  return useQuery({
    queryKey: consentKeys.all,
    queryFn: async () => {
      const response = await getConsents();
      return response;
    },
  });
}

export function useGrantConsent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (data: GrantConsentRequest) => {
      const response = await grantConsent(data);
      return response;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: consentKeys.all });
      toast.success("Consent granted successfully");
    },
    onError: (err) => {
      toast.error(err instanceof Error ? err.message : "Failed to grant consent");
    },
  });
}

export function useRevokeConsent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ id, target_domain }: { id: string; target_domain: string }) => {
      const response = await revokeConsent(id, target_domain);
      return response;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: consentKeys.all });
      toast.success("Consent revoked successfully");
    },
    onError: (err) => {
      toast.error(err instanceof Error ? err.message : "Failed to revoke consent");
    },
  });
}
