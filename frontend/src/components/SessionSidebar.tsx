'use client';

import { useState } from 'react';
import { useScanSessionStore, getModuleLabel, formatDuration } from '@/lib/scanSessionStore';
import {
  Clock, CheckCircle2, XCircle, Loader2, Circle, X, Trash2, ChevronRight, ChevronDown, ChevronUp, Globe
} from 'lucide-react';
import { useRouter, usePathname } from 'next/navigation';

const moduleRoutes: Record<string, string> = {
  scanner: '/scanner',
  investigator: '/investigator',
  'api-discovery': '/api-discovery',
  services: '/services',
  audit: '/audit',
  forms: '/forms',
  proxy: '/proxy',
};

function StatusIcon({ status }: { status: string }) {
  switch (status) {
    case 'success':
      return <CheckCircle2 size={13} color="#86efac" />;
    case 'loading':
      return <Loader2 size={13} color="#fcd34d" style={{ animation: 'spin 1s linear infinite' }} />;
    case 'error':
      return <XCircle size={13} color="#fca5a5" />;
    default:
      return <Circle size={13} color="var(--text-muted)" />;
  }
}

export default function SessionSidebar() {
  const store = useScanSessionStore();
  const session = store.activeSession;
  const router = useRouter();
  const pathname = usePathname();
  const [isMinimized, setIsMinimized] = useState(false);

  if (!session) return null;

  const moduleEntries = Object.entries(session.moduleResults);
  const completedCount = moduleEntries.filter(([, r]) => r.status === 'success').length;
  const elapsed = formatDuration(Date.now() - session.startTime);

  if (isMinimized) {
    return (
      <button 
        onClick={() => setIsMinimized(false)}
        style={{
          position: 'fixed',
          bottom: 20,
          right: 20,
          background: 'rgba(8, 12, 28, 0.92)',
          backdropFilter: 'blur(24px)',
          border: '1px solid rgba(0, 212, 255, 0.2)',
          borderRadius: 12,
          padding: '10px 16px',
          zIndex: 1000,
          boxShadow: '0 8px 30px rgba(0, 0, 0, 0.5), 0 0 20px rgba(0, 212, 255, 0.05)',
          display: 'flex',
          alignItems: 'center',
          gap: 12,
          cursor: 'pointer',
          color: 'var(--text-primary)',
          fontFamily: 'var(--font-sans)',
          transition: 'all 0.2s',
        }}
        onMouseEnter={e => e.currentTarget.style.background = 'rgba(15, 20, 40, 0.95)'}
        onMouseLeave={e => e.currentTarget.style.background = 'rgba(8, 12, 28, 0.92)'}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
          <Globe size={14} color="#00d4ff" />
          <span style={{ fontSize: '0.8rem', fontWeight: 600 }}>Active Session</span>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          <span style={{ fontSize: '0.75rem', color: 'var(--text-muted)', fontFamily: 'var(--font-mono)' }}>
            {completedCount}/{moduleEntries.length}
          </span>
          <ChevronUp size={14} color="var(--text-muted)" />
        </div>
      </button>
    );
  }

  return (
    <div style={{
      position: 'fixed',
      bottom: 20,
      right: 20,
      width: 280,
      background: 'rgba(8, 12, 28, 0.92)',
      backdropFilter: 'blur(24px)',
      border: '1px solid rgba(100, 116, 139, 0.12)',
      borderRadius: 16,
      padding: 0,
      zIndex: 1000,
      boxShadow: '0 8px 40px rgba(0, 0, 0, 0.5), 0 0 60px rgba(0, 212, 255, 0.03)',
      overflow: 'hidden',
      fontFamily: 'var(--font-sans)',
      transition: 'all 0.2s',
    }}>
      {/* Header */}
      <div style={{
        padding: '14px 16px 10px',
        borderBottom: '1px solid rgba(100, 116, 139, 0.08)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
      }}>
        <div>
          <div style={{
            fontSize: '0.68rem',
            fontWeight: 700,
            textTransform: 'uppercase',
            letterSpacing: '0.08em',
            color: 'var(--text-muted)',
            marginBottom: 4,
          }}>Active Session</div>
          <div style={{
            fontSize: '0.82rem',
            fontFamily: 'var(--font-mono)',
            color: 'var(--accent-cyan)',
            maxWidth: 180,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
            title={session.targetUrl}
          >
            {session.targetUrl.replace(/^https?:\/\//, '')}
          </div>
        </div>
        <div style={{ display: 'flex', gap: 4 }}>
          <button
            onClick={() => setIsMinimized(true)}
            style={{
              background: 'transparent',
              border: 'none',
              color: 'var(--text-muted)',
              cursor: 'pointer',
              padding: 4,
              borderRadius: 4,
              display: 'flex',
              alignItems: 'center',
              transition: 'color 0.15s, background 0.15s',
            }}
            title="Minimize"
            onMouseEnter={e => { e.currentTarget.style.color = '#fff'; e.currentTarget.style.background = 'rgba(255,255,255,0.1)'; }}
            onMouseLeave={e => { e.currentTarget.style.color = 'var(--text-muted)'; e.currentTarget.style.background = 'transparent'; }}
          >
            <ChevronDown size={14} />
          </button>
          <button
            onClick={() => store.closeSession(session.id)}
            style={{
              background: 'transparent',
              border: 'none',
              color: 'var(--text-muted)',
              cursor: 'pointer',
              padding: 4,
              borderRadius: 4,
              display: 'flex',
              alignItems: 'center',
              transition: 'color 0.15s, background 0.15s',
            }}
            title="Close session"
            onMouseEnter={e => { e.currentTarget.style.color = '#fca5a5'; e.currentTarget.style.background = 'rgba(239, 68, 68, 0.1)'; }}
            onMouseLeave={e => { e.currentTarget.style.color = 'var(--text-muted)'; e.currentTarget.style.background = 'transparent'; }}
          >
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Modules */}
      <div style={{ padding: '8px 8px' }}>
        {moduleEntries.length === 0 ? (
          <div style={{
            padding: '12px 8px',
            fontSize: '0.78rem',
            color: 'var(--text-muted)',
            textAlign: 'center',
            fontStyle: 'italic',
          }}>
            No modules scanned yet
          </div>
        ) : (
          moduleEntries.map(([moduleId, result]) => {
            const route = moduleRoutes[moduleId];
            const isActive = pathname === route;
            return (
              <button
                key={moduleId}
                onClick={() => route && router.push(route)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 10,
                  width: '100%',
                  padding: '8px 10px',
                  background: isActive ? 'rgba(0, 212, 255, 0.06)' : 'transparent',
                  border: 'none',
                  borderRadius: 8,
                  cursor: route ? 'pointer' : 'default',
                  transition: 'background 0.15s',
                  color: 'var(--text-secondary)',
                  textAlign: 'left',
                  fontSize: '0.78rem',
                  fontFamily: 'var(--font-sans)',
                }}
                onMouseEnter={e => {
                  if (!isActive) e.currentTarget.style.background = 'rgba(255,255,255,0.03)';
                }}
                onMouseLeave={e => {
                  if (!isActive) e.currentTarget.style.background = 'transparent';
                }}
              >
                <StatusIcon status={result.status} />
                <span style={{ flex: 1, fontWeight: isActive ? 600 : 400 }}>
                  {getModuleLabel(moduleId)}
                </span>
                {result.status === 'success' && (
                  <ChevronRight size={12} color="var(--text-muted)" />
                )}
              </button>
            );
          })
        )}
      </div>

      {/* Footer */}
      <div style={{
        padding: '8px 16px 12px',
        borderTop: '1px solid rgba(100, 116, 139, 0.06)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
      }}>
        <div style={{
          display: 'flex',
          alignItems: 'center',
          gap: 6,
          fontSize: '0.7rem',
          color: 'var(--text-muted)',
        }}>
          <Clock size={11} />
          {elapsed}
        </div>
        <div style={{
          fontSize: '0.7rem',
          color: 'var(--text-muted)',
        }}>
          {completedCount} / {moduleEntries.length} modules
        </div>
      </div>
    </div>
  );
}
