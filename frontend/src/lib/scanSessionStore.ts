import { create } from 'zustand';
import { persist } from 'zustand/middleware';

// ── Stream State (SSE entegrasyonu için) ──
export interface StreamEvent {
  timestamp: string;
  type: string;
  message: string;
  raw?: unknown;
}

export interface StreamState {
  sessionId: string;
  moduleId: string;
  status: 'idle' | 'streaming' | 'completed' | 'error';
  events: StreamEvent[];
  result: unknown | null;
  error: string | null;
  startTime: number;
  endTime?: number;
}

// Her modül için sonuç tipi
export interface ModuleResult {
  moduleId: string;
  moduleName: string;
  targetUrl: string;
  timestamp: number;
  result: unknown;
  status: 'idle' | 'loading' | 'success' | 'error';
  error?: string;
}

// Aktif scan session
export interface ScanSession {
  id: string;
  targetUrl: string;
  startTime: number;
  moduleResults: Record<string, ModuleResult>;
  streamStates: Record<string, StreamState>;
  isComplete: boolean;
}

interface ScanSessionState {
  // Aktif session
  activeSession: ScanSession | null;
  
  // Son 20 session (geçmiş)
  recentSessions: ScanSession[];
  
  // ── Session Actions ──
  createSession: (targetUrl: string) => string;
  closeSession: (sessionId: string) => void;
  deleteSession: (sessionId: string) => void;
  restoreSession: (sessionId: string) => void;
  clearAllSessions: () => void;
  getSession: (sessionId: string) => ScanSession | undefined;

  // ── Module Result Actions ──
  setModuleResult: (sessionId: string, moduleId: string, result: Partial<ModuleResult>) => void;
  setModuleLoading: (sessionId: string, moduleId: string) => void;
  setModuleError: (sessionId: string, moduleId: string, error: string) => void;
  getModuleResult: (sessionId: string, moduleId: string) => ModuleResult | undefined;

  // ── Stream State Actions (SSE entegrasyonu) ──
  setStreamState: (sessionId: string, moduleId: string, state: Partial<StreamState>) => void;
  appendStreamEvent: (sessionId: string, moduleId: string, event: StreamEvent) => void;
  getStreamState: (sessionId: string, moduleId: string) => StreamState | undefined;
}

// ── Helper: update a session in-place ──
function updateSessionInState(
  state: Pick<ScanSessionState, 'activeSession' | 'recentSessions'>,
  sessionId: string,
  updater: (session: ScanSession) => ScanSession,
) {
  return {
    activeSession: state.activeSession?.id === sessionId
      ? updater(state.activeSession)
      : state.activeSession,
    recentSessions: state.recentSessions.map(s =>
      s.id === sessionId ? updater(s) : s,
    ),
  };
}

export const useScanSessionStore = create<ScanSessionState>()(
  persist(
    (set, get) => ({
      activeSession: null,
      recentSessions: [],

      // ════════════════════════════════════════
      //  SESSION LIFECYCLE
      // ════════════════════════════════════════

      createSession: (targetUrl: string) => {
        const sessionId = `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
        const newSession: ScanSession = {
          id: sessionId,
          targetUrl,
          startTime: Date.now(),
          moduleResults: {},
          streamStates: {},
          isComplete: false,
        };
        
        set((state) => ({
          activeSession: newSession,
          recentSessions: [newSession, ...state.recentSessions].slice(0, 20),
        }));
        
        return sessionId;
      },

      closeSession: (sessionId: string) => {
        set((state) => {
          if (state.activeSession?.id === sessionId) {
            const completedSession: ScanSession = {
              ...state.activeSession,
              isComplete: true,
              // Clear stream events to save localStorage space
              streamStates: Object.fromEntries(
                Object.entries(state.activeSession.streamStates).map(([k, v]) => [
                  k,
                  { ...v, events: [], status: 'completed' as const },
                ]),
              ),
            };
            return {
              activeSession: null,
              recentSessions: [
                completedSession,
                ...state.recentSessions.filter(s => s.id !== sessionId),
              ].slice(0, 20),
            };
          }
          return state;
        });
      },

      deleteSession: (sessionId: string) => {
        set((state) => ({
          activeSession: state.activeSession?.id === sessionId ? null : state.activeSession,
          recentSessions: state.recentSessions.filter(s => s.id !== sessionId),
        }));
      },

      restoreSession: (sessionId: string) => {
        set((state) => {
          const target = state.recentSessions.find(s => s.id === sessionId);
          if (!target) return state;

          // Close currently active session first
          const oldActive = state.activeSession;
          const remaining = state.recentSessions.filter(s => s.id !== sessionId);
          const sessions = oldActive
            ? [{ ...oldActive, isComplete: true }, ...remaining]
            : remaining;

          return {
            activeSession: { ...target, isComplete: false },
            recentSessions: sessions.slice(0, 20),
          };
        });
      },

      clearAllSessions: () => {
        set({ activeSession: null, recentSessions: [] });
      },

      getSession: (sessionId: string) => {
        const state = get();
        if (state.activeSession?.id === sessionId) return state.activeSession;
        return state.recentSessions.find(s => s.id === sessionId);
      },

      // ════════════════════════════════════════
      //  MODULE RESULTS
      // ════════════════════════════════════════

      setModuleResult: (sessionId: string, moduleId: string, result: Partial<ModuleResult>) => {
        set((state) =>
          updateSessionInState(state, sessionId, (session) => ({
            ...session,
            moduleResults: {
              ...session.moduleResults,
              [moduleId]: {
                ...session.moduleResults[moduleId],
                ...result,
                moduleId,
                timestamp: Date.now(),
                status: 'success' as const,
              } as ModuleResult,
            },
          })),
        );
      },

      setModuleLoading: (sessionId: string, moduleId: string) => {
        set((state) =>
          updateSessionInState(state, sessionId, (session) => ({
            ...session,
            moduleResults: {
              ...session.moduleResults,
              [moduleId]: {
                ...session.moduleResults[moduleId],
                moduleId,
                moduleName: moduleId,
                targetUrl: session.targetUrl,
                timestamp: Date.now(),
                result: null,
                status: 'loading' as const,
              } as ModuleResult,
            },
          })),
        );
      },

      setModuleError: (sessionId: string, moduleId: string, error: string) => {
        set((state) =>
          updateSessionInState(state, sessionId, (session) => ({
            ...session,
            moduleResults: {
              ...session.moduleResults,
              [moduleId]: {
                ...session.moduleResults[moduleId],
                moduleId,
                moduleName: moduleId,
                targetUrl: session.targetUrl,
                timestamp: Date.now(),
                result: null,
                status: 'error' as const,
                error,
              } as ModuleResult,
            },
          })),
        );
      },

      getModuleResult: (sessionId: string, moduleId: string) => {
        const session = get().getSession(sessionId);
        return session?.moduleResults[moduleId];
      },

      // ════════════════════════════════════════
      //  STREAM STATE (SSE)
      // ════════════════════════════════════════

      setStreamState: (sessionId: string, moduleId: string, partial: Partial<StreamState>) => {
        set((state) =>
          updateSessionInState(state, sessionId, (session) => {
            const defaults: StreamState = {
              sessionId, moduleId, status: 'idle', events: [], result: null, error: null, startTime: Date.now(),
            };
            const merged: StreamState = Object.assign(defaults, session.streamStates[moduleId], partial, { sessionId, moduleId });
            return {
              ...session,
              streamStates: { ...session.streamStates, [moduleId]: merged },
            };
          }),
        );
      },

      appendStreamEvent: (sessionId: string, moduleId: string, event: StreamEvent) => {
        set((state) =>
          updateSessionInState(state, sessionId, (session) => {
            const existing = session.streamStates[moduleId];
            const defaults: StreamState = {
              sessionId, moduleId, status: 'streaming', events: [], result: null, error: null, startTime: Date.now(),
            };
            const merged: StreamState = Object.assign(defaults, existing, {
              sessionId, moduleId,
              events: [...(existing?.events || []), event].slice(-100),
            });
            return {
              ...session,
              streamStates: { ...session.streamStates, [moduleId]: merged },
            };
          }),
        );
      },

      getStreamState: (sessionId: string, moduleId: string) => {
        const session = get().getSession(sessionId);
        return session?.streamStates[moduleId];
      },
    }),
    {
      name: 'limma-scan-sessions', // localStorage key
      partialize: (state) => ({
        // Persist recent sessions (with results) but NOT active session
        // Active session is ephemeral — only lives while the tab is open
        recentSessions: state.recentSessions.map(s => ({
          ...s,
          // Strip stream events from persisted data (they can be large)
          streamStates: Object.fromEntries(
            Object.entries(s.streamStates).map(([k, v]) => [
              k,
              { ...v, events: [] },
            ]),
          ),
        })),
      }),
    },
  ),
);

// ── Helper hooks ──

/** Get module result from active session */
export function useModuleResult(moduleId: string) {
  const store = useScanSessionStore();
  const session = store.activeSession;
  return session?.moduleResults[moduleId];
}

/** Get active session */
export function useActiveSession() {
  return useScanSessionStore((state) => state.activeSession);
}

/** Module display labels */
export function getModuleLabel(moduleId: string): string {
  const labels: Record<string, string> = {
    scanner: 'Website Scanner',
    investigator: 'Server Investigator',
    'api-discovery': 'API Discovery',
    services: 'Service Collector',
    audit: 'Security Audit',
    forms: 'Form Mapper',
    proxy: 'Proxy Tester',
  };
  return labels[moduleId] || moduleId;
}

/** Format duration (ms) to human-readable string */
export function formatDuration(ms: number): string {
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ${seconds % 60}s`;
  const hours = Math.floor(minutes / 60);
  return `${hours}h ${minutes % 60}m`;
}
