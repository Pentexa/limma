'use client';

import { useScanSessionStore, getModuleLabel, formatDuration } from '@/lib/scanSessionStore';
import type { ScanSession } from '@/lib/scanSessionStore';
import {
  History, Trash2, RotateCcw, CheckCircle2, XCircle, Clock, Globe, AlertTriangle
} from 'lucide-react';

function SessionCard({ session, onRestore, onDelete }: {
  session: ScanSession;
  onRestore: () => void;
  onDelete: () => void;
}) {
  const modules = Object.entries(session.moduleResults);
  const successCount = modules.filter(([, r]) => r.status === 'success').length;
  const errorCount = modules.filter(([, r]) => r.status === 'error').length;
  const elapsed = formatDuration(Date.now() - session.startTime);
  const dateStr = new Date(session.startTime).toLocaleString();

  return (
    <div className="glass-card" style={{
      padding: 0,
      overflow: 'hidden',
      transition: 'border-color 0.2s, box-shadow 0.2s',
    }}>
      {/* Header */}
      <div style={{
        padding: '16px 20px',
        display: 'flex',
        alignItems: 'flex-start',
        justifyContent: 'space-between',
        gap: 12,
      }}>
        <div style={{ flex: 1, minWidth: 0 }}>
          <div style={{
            fontFamily: 'var(--font-mono)',
            fontSize: '0.88rem',
            color: 'var(--accent-cyan)',
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }} title={session.targetUrl}>
            {session.targetUrl}
          </div>
          <div style={{
            display: 'flex',
            gap: 16,
            marginTop: 6,
            fontSize: '0.75rem',
            color: 'var(--text-muted)',
          }}>
            <span style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
              <Clock size={12} /> {dateStr}
            </span>
            <span>{elapsed} ago</span>
          </div>
        </div>

        <div style={{ display: 'flex', gap: 6, flexShrink: 0 }}>
          <button
            onClick={onRestore}
            style={{
              display: 'flex', alignItems: 'center', gap: 6,
              padding: '6px 14px', fontSize: '0.75rem', fontWeight: 600,
              background: 'rgba(0, 212, 255, 0.08)', color: '#7dd3fc',
              border: '1px solid rgba(0, 212, 255, 0.2)', borderRadius: 8,
              cursor: 'pointer', transition: 'all 0.15s',
            }}
            onMouseEnter={e => (e.currentTarget.style.background = 'rgba(0, 212, 255, 0.14)')}
            onMouseLeave={e => (e.currentTarget.style.background = 'rgba(0, 212, 255, 0.08)')}
          >
            <RotateCcw size={12} /> Restore
          </button>
          <button
            onClick={onDelete}
            style={{
              display: 'flex', alignItems: 'center',
              padding: '6px 10px',
              background: 'rgba(239, 68, 68, 0.06)', color: '#fca5a5',
              border: '1px solid rgba(239, 68, 68, 0.15)', borderRadius: 8,
              cursor: 'pointer', transition: 'all 0.15s',
            }}
            onMouseEnter={e => (e.currentTarget.style.background = 'rgba(239, 68, 68, 0.12)')}
            onMouseLeave={e => (e.currentTarget.style.background = 'rgba(239, 68, 68, 0.06)')}
            title="Delete session"
          >
            <Trash2 size={13} />
          </button>
        </div>
      </div>

      {/* Module chips */}
      {modules.length > 0 && (
        <div style={{
          padding: '0 20px 16px',
          display: 'flex',
          flexWrap: 'wrap',
          gap: 6,
        }}>
          {modules.map(([moduleId, result]) => (
            <span key={moduleId} style={{
              display: 'inline-flex', alignItems: 'center', gap: 5,
              padding: '3px 10px',
              fontSize: '0.7rem', fontWeight: 600,
              background: result.status === 'success' ? 'rgba(16, 185, 129, 0.08)' :
                result.status === 'error' ? 'rgba(239, 68, 68, 0.08)' : 'rgba(100, 116, 139, 0.08)',
              color: result.status === 'success' ? '#86efac' :
                result.status === 'error' ? '#fca5a5' : 'var(--text-muted)',
              border: `1px solid ${result.status === 'success' ? 'rgba(16, 185, 129, 0.15)' :
                result.status === 'error' ? 'rgba(239, 68, 68, 0.15)' : 'rgba(100, 116, 139, 0.1)'}`,
              borderRadius: 20,
              whiteSpace: 'nowrap',
            }}>
              {result.status === 'success' ? <CheckCircle2 size={10} /> :
                result.status === 'error' ? <XCircle size={10} /> : null}
              {getModuleLabel(moduleId)}
            </span>
          ))}
        </div>
      )}

      {/* Stats footer */}
      <div style={{
        padding: '10px 20px',
        borderTop: '1px solid rgba(100, 116, 139, 0.06)',
        display: 'flex',
        gap: 16,
        fontSize: '0.72rem',
        color: 'var(--text-muted)',
      }}>
        <span>{modules.length} module{modules.length !== 1 ? 's' : ''}</span>
        <span style={{ color: '#86efac' }}>{successCount} passed</span>
        {errorCount > 0 && <span style={{ color: '#fca5a5' }}>{errorCount} failed</span>}
      </div>
    </div>
  );
}

export default function SessionsPage() {
  const { recentSessions, deleteSession, restoreSession, clearAllSessions, activeSession } = useScanSessionStore();

  // Don't show active session in history
  const historySessions = recentSessions.filter(s => s.id !== activeSession?.id);

  return (
    <div className="fade-in">
      <div className="page-header">
        <h1 className="page-title">Session History</h1>
        <p className="page-subtitle">
          Browse past scan sessions, restore results, and manage your scanning history
        </p>
      </div>

      {/* Active session banner */}
      {activeSession && (
        <div className="glass-card mb-6" style={{
          display: 'flex', alignItems: 'center', gap: 12,
          padding: '14px 20px',
          borderLeft: '3px solid rgba(56, 189, 248, 0.5)',
          background: 'rgba(56, 189, 248, 0.04)',
        }}>
          <Globe size={16} color="#7dd3fc" />
          <div style={{ flex: 1 }}>
            <div style={{ fontSize: '0.85rem', color: '#7dd3fc', fontWeight: 600 }}>Active Session</div>
            <div style={{ fontSize: '0.78rem', fontFamily: 'var(--font-mono)', color: 'var(--text-secondary)' }}>
              {activeSession.targetUrl}
            </div>
          </div>
          <div style={{ fontSize: '0.72rem', color: 'var(--text-muted)' }}>
            {Object.keys(activeSession.moduleResults).length} modules
          </div>
        </div>
      )}

      {/* Controls */}
      {historySessions.length > 0 && (
        <div style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: 16,
        }}>
          <div style={{ fontSize: '0.82rem', color: 'var(--text-muted)' }}>
            {historySessions.length} session{historySessions.length !== 1 ? 's' : ''}
          </div>
          <button
            onClick={clearAllSessions}
            style={{
              display: 'flex', alignItems: 'center', gap: 6,
              padding: '6px 14px', fontSize: '0.75rem', fontWeight: 600,
              background: 'rgba(239, 68, 68, 0.06)', color: '#fca5a5',
              border: '1px solid rgba(239, 68, 68, 0.15)', borderRadius: 8,
              cursor: 'pointer', transition: 'all 0.15s',
            }}
          >
            <Trash2 size={12} /> Clear All
          </button>
        </div>
      )}

      {/* Sessions list */}
      <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
        {historySessions.map(session => (
          <SessionCard
            key={session.id}
            session={session}
            onRestore={() => restoreSession(session.id)}
            onDelete={() => deleteSession(session.id)}
          />
        ))}
      </div>

      {/* Empty state */}
      {historySessions.length === 0 && (
        <div className="empty-state">
          <div className="empty-state-icon"><History size={36} /></div>
          <div className="empty-state-title">No Session History</div>
          <div className="empty-state-text">
            Completed scan sessions will appear here. Use any scanner module to start a session.
          </div>
        </div>
      )}
    </div>
  );
}
