import type { PropsWithChildren } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fetchFindings, fetchMasterReportFindings } from "../api/finding-api";
import { useScans } from "@/entities/scan/model/use-scans";
import { useGlobalFindings } from "./use-findings";

vi.mock("../api/finding-api", () => ({
  fetchFindings: vi.fn(),
  fetchFilteredFindings: vi.fn(),
  fetchFinding: vi.fn(),
  fetchMasterReportFindings: vi.fn(),
}));

vi.mock("@/entities/scan/model/use-scans", () => ({
  useScans: vi.fn(),
}));

const mockUseScans = vi.mocked(useScans);
const mockFetchFindings = vi.mocked(fetchFindings);
const mockFetchMasterReportFindings = vi.mocked(fetchMasterReportFindings);

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return function Wrapper({ children }: PropsWithChildren) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
}

describe("useGlobalFindings", () => {
  beforeEach(() => {
    mockFetchFindings.mockResolvedValue([]);
    mockFetchMasterReportFindings.mockResolvedValue([]);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  it("loads active findings without requesting the master report", async () => {
    mockUseScans.mockReturnValue({
      data: [{ id: "scan-1", targetUrl: "https://example.test", status: "running" }],
      isLoading: false,
      isFetching: false,
    } as unknown as ReturnType<typeof useScans>);

    const { result } = renderHook(() => useGlobalFindings(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(mockFetchFindings).toHaveBeenCalledWith("scan-1");
    expect(mockFetchMasterReportFindings).not.toHaveBeenCalled();
  });

  it("keeps finding queries disabled when there is no scan", async () => {
    mockUseScans.mockReturnValue({
      data: [],
      isLoading: false,
      isFetching: false,
    } as unknown as ReturnType<typeof useScans>);

    const { result } = renderHook(() => useGlobalFindings(), { wrapper: createWrapper() });

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.data).toEqual([]);
    expect(mockFetchFindings).not.toHaveBeenCalled();
    expect(mockFetchMasterReportFindings).not.toHaveBeenCalled();
  });
});
