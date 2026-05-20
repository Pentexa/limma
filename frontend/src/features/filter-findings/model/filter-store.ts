import { create } from "zustand";
import type { Severity } from "@/shared/types/common";
import type { DetectorType, VerificationStatus } from "@/entities/finding/model/types";

interface FilterState {
  severity: Severity[];
  detectors: DetectorType[];
  verification: VerificationStatus[];
  searchQuery: string;

  setSeverity: (severity: Severity[]) => void;
  setDetectors: (detectors: DetectorType[]) => void;
  setVerification: (verification: VerificationStatus[]) => void;
  setSearchQuery: (query: string) => void;
  clearFilters: () => void;
}

export const useFilterStore = create<FilterState>((set) => ({
  severity: [],
  detectors: [],
  verification: [],
  searchQuery: "",

  setSeverity: (severity) => set({ severity }),
  setDetectors: (detectors) => set({ detectors }),
  setVerification: (verification) => set({ verification }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  clearFilters: () => set({ severity: [], detectors: [], verification: [], searchQuery: "" }),
}));
