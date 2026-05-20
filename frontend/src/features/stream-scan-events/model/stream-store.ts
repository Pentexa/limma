import { create } from "zustand";
import type { ConnectionStatus } from "@/shared/types/common";
import type { SSEMessage } from "@/shared/api/sse-client";

/** Maximum events to keep in buffer */
const MAX_EVENTS = 500;

/** Local scan state — updated immediately on user action, independent of polling */
export type LocalScanState = "idle" | "starting" | "running" | "completed";

export interface StreamEvent extends SSEMessage {
  id: string;
}

interface StreamState {
  /** Current SSE connection status */
  connectionStatus: ConnectionStatus;
  /** Buffered events (newest first) */
  events: StreamEvent[];
  /** Total events received since connection */
  totalEvents: number;
  /** Whether the stream is paused (UI only, SSE still connected) */
  isPaused: boolean;

  /** Local scan state — set immediately, not dependent on polling */
  localScanState: LocalScanState;
  /** Target URL of the current/last scan */
  localScanTarget: string | null;
  /** ID of the locally started scan */
  localScanId: string | null;

  // Actions
  setConnectionStatus: (status: ConnectionStatus) => void;
  addEvent: (event: SSEMessage) => void;
  clearEvents: () => void;
  togglePause: () => void;

  /** Mark a scan as starting (called immediately on user click) */
  setScanStarting: (targetUrl: string) => void;
  /** Mark a scan as actively running (called when backend confirms) */
  setScanRunning: (scanId?: string) => void;
  /** Mark a scan as completed */
  setScanCompleted: () => void;
  /** Reset scan state to idle */
  setScanIdle: () => void;
}

let eventCounter = 0;
const MAX_COUNTER = 1_000_000; // Reset counter to prevent Number overflow

export const useStreamStore = create<StreamState>((set) => ({
  connectionStatus: "disconnected",
  events: [],
  totalEvents: 0,
  isPaused: false,
  localScanState: "idle",
  localScanTarget: null,
  localScanId: null,

  setConnectionStatus: (status) => set({ connectionStatus: status }),

  addEvent: (event) =>
    set((state) => {
      if (state.isPaused) {
        return { totalEvents: state.totalEvents + 1 };
      }
      eventCounter = (eventCounter + 1) % MAX_COUNTER;
      const newEvent: StreamEvent = {
        ...event,
        id: `evt-${eventCounter}`,
      };
      const events = [newEvent, ...state.events].slice(0, MAX_EVENTS);

      // If we receive events while "starting", auto-promote to "running"
      const localScanState =
        state.localScanState === "starting" ? "running" : state.localScanState;

      return { events, totalEvents: state.totalEvents + 1, localScanState };
    }),

  clearEvents: () => {
    eventCounter = 0;
    return set({ events: [], totalEvents: 0 });
  },

  togglePause: () => set((state) => ({ isPaused: !state.isPaused })),

  setScanStarting: (targetUrl) =>
    set({ localScanState: "starting", localScanTarget: targetUrl, localScanId: null }),

  setScanRunning: (scanId) => set((state) => ({ 
    localScanState: "running", 
    localScanId: scanId !== undefined ? scanId : state.localScanId 
  })),

  setScanCompleted: () =>
    set({ localScanState: "completed" }),

  setScanIdle: () =>
    set({ localScanState: "idle", localScanTarget: null, localScanId: null }),
}));
