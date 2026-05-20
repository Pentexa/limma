'use client';

/**
 * Example Tool Container — Template
 * 
 * Copy this file and customize it to create a new tool.
 * Then register it in lib/tool-registry.ts.
 * 
 * See docs/NEW_TOOL_GUIDE.md for full instructions.
 */

import { useState, useCallback } from 'react';
import UrlInput from '@/components/UrlInput';
import ErrorAlert from '@/components/ErrorAlert';
import EmptyState from '@/components/EmptyState';
import { useScanSessionStore } from '@/lib/scanSessionStore';
import { useLiveEventsStore } from '@/lib/stores/live-events.store';
import { useWorkspaceSelectionStore } from '@/lib/stores/workspace-selection.store';
import { Wrench } from 'lucide-react';

export function ExampleToolContainer() {
  const store = useScanSessionStore();
  const addConsoleLine = useLiveEventsStore((s) => s.addConsoleLine);
  const [result, setResult] = useState<unknown>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleScan = useCallback(async (url: string) => {
    setLoading(true);
    setError(null);
    setResult(null);

    // Create/reuse scan session
    const current = store.activeSession;
    if (!current || current.targetUrl !== url) {
      if (current) store.closeSession(current.id);
      store.createSession(url);
    }
    const sessionId = useScanSessionStore.getState().activeSession!.id;
    store.setModuleLoading(sessionId, 'example-tool');

    // Log to runtime console
    addConsoleLine({
      timestamp: new Date().toISOString(),
      level: 'info',
      message: `Example tool scanning: ${url}`,
      source: 'example-tool',
    });

    try {
      // TODO: Replace with actual API call
      // const res = await myToolApi(url);
      const res = { status: 'ok', url };
      
      setResult(res);
      store.setModuleResult(sessionId, 'example-tool', {
        moduleId: 'example-tool',
        moduleName: 'Example Tool',
        targetUrl: url,
        result: res,
        status: 'success',
      });

      addConsoleLine({
        timestamp: new Date().toISOString(),
        level: 'info',
        message: 'Example tool completed',
        source: 'example-tool',
      });
    } catch (e) {
      const msg = e instanceof Error ? e.message : 'Tool execution failed';
      setError(msg);
      store.setModuleError(sessionId, 'example-tool', msg);

      addConsoleLine({
        timestamp: new Date().toISOString(),
        level: 'error',
        message: `Example tool error: ${msg}`,
        source: 'example-tool',
      });
    } finally {
      setLoading(false);
    }
  }, [store, addConsoleLine]);

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">Example Tool</h1>
        <p className="page-subtitle">
          This is a template tool. Replace this with your tool description.
        </p>
      </div>

      <UrlInput onSubmit={handleScan} loading={loading} buttonLabel="Run Tool" />

      {loading && (
        <div className="loading-overlay">
          <div className="loading-spinner" />
          <div className="loading-text">Running example tool...</div>
        </div>
      )}

      {error && <ErrorAlert title="Tool Error" message={error} />}

      {result && (
        <div className="fade-in">
          <div className="glass-card" style={{ padding: 20 }}>
            <pre style={{
              fontFamily: 'var(--font-mono)',
              fontSize: '0.78rem',
              color: 'var(--text-primary)',
              whiteSpace: 'pre-wrap',
            }}>
              {JSON.stringify(result, null, 2)}
            </pre>
          </div>
        </div>
      )}

      {!result && !loading && !error && (
        <EmptyState
          icon={<Wrench size={36} />}
          title="Example Tool"
          description="Enter a URL to run the example tool. This is a template — replace with your actual tool logic."
        />
      )}
    </div>
  );
}
