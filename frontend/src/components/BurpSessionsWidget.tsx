'use client';

import { useState, useEffect } from 'react';
import { Activity, Shield, RefreshCw } from 'lucide-react';

interface BurpSession {
  session_id: string;
  status: string; // 'Connected', 'Syncing', 'Disconnected'
  target_url: string;
  imported_traffic_count: number;
  exported_findings_count: number;
  connected_at: string;
  last_heartbeat: string;
}

export default function BurpSessionsWidget() {
  const [sessions, setSessions] = useState<BurpSession[]>([]);
  const [loading, setLoading] = useState(true);

  const fetchSessions = async () => {
    try {
      const res = await fetch('http://localhost:8900/api/burp/sessions');
      if (res.ok) {
        const data = await res.json();
        setSessions(data);
      }
    } catch (e) {
      console.error('Failed to fetch burp sessions', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchSessions();
    const interval = setInterval(fetchSessions, 15000); // poll every 15s to respect backend rate limits
    return () => clearInterval(interval);
  }, []);

  if (loading && sessions.length === 0) {
    return null; // or a tiny loader
  }

  if (sessions.length === 0 && !loading) {
    return (
      <div className="glass-card mb-6" style={{
        borderLeft: '3px solid var(--accent-violet)',
        background: 'linear-gradient(135deg, rgba(139, 92, 246, 0.05) 0%, transparent 100%)'
      }}>
        <div className="flex items-center gap-2 mb-3">
          <Activity size={18} color="var(--accent-violet)" />
          <h3 style={{ fontFamily: 'var(--font-display)', fontWeight: 700, fontSize: '0.95rem' }}>Burp Suite Integration (Not Connected)</h3>
        </div>
        <p style={{ fontSize: '0.85rem', color: 'var(--text-secondary)', marginBottom: 12, lineHeight: 1.6 }}>
          <strong>Did you know?</strong> You can combine your manual pentesting with LIMMA's AI Rule Engine. 
          By connecting Burp Suite, LIMMA will silently analyze every HTTP request you make in real-time and instantly notify you of hidden vulnerabilities, with <strong>Zero False Positives</strong>.
        </p>

        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: 12, marginTop: 16 }}>
          <div style={{ background: 'rgba(0,0,0,0.2)', padding: 16, borderRadius: 'var(--radius-sm)', border: '1px solid rgba(255,255,255,0.05)' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
              <div style={{ background: 'var(--accent-violet)', color: '#fff', width: 20, height: 20, borderRadius: '50%', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '0.75rem', fontWeight: 'bold' }}>1</div>
              <div style={{ fontWeight: 600, fontSize: '0.85rem', color: 'var(--text-primary)' }}>Get Burp Suite</div>
            </div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', lineHeight: 1.5 }}>
              If you don't have it, download <strong>Burp Suite Community Edition</strong> (Free) from PortSwigger's website and install it.
            </div>
          </div>
          
          <div style={{ background: 'rgba(0,0,0,0.2)', padding: 16, borderRadius: 'var(--radius-sm)', border: '1px solid rgba(255,255,255,0.05)' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
              <div style={{ background: 'var(--accent-violet)', color: '#fff', width: 20, height: 20, borderRadius: '50%', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '0.75rem', fontWeight: 'bold' }}>2</div>
              <div style={{ fontWeight: 600, fontSize: '0.85rem', color: 'var(--text-primary)' }}>Install LIMMA Plugin</div>
            </div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', lineHeight: 1.5, marginBottom: 12 }}>
              Download the official plugin directly from our servers. In Burp Suite, navigate to <strong>Extender &gt; Extensions &gt; Add</strong>. Select Java and choose this JAR file.
            </div>
            <a 
              href="/downloads/limma-burp-plugin.jar" 
              download
              style={{
                display: 'inline-flex',
                alignItems: 'center',
                gap: 6,
                background: 'var(--accent-violet)',
                color: 'white',
                padding: '6px 12px',
                borderRadius: 'var(--radius-sm)',
                fontSize: '0.75rem',
                fontWeight: 600,
                textDecoration: 'none',
                transition: 'background 0.2s',
              }}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
              Download Plugin (.jar)
            </a>
          </div>

          <div style={{ background: 'rgba(0,0,0,0.2)', padding: 16, borderRadius: 'var(--radius-sm)', border: '1px solid rgba(255,255,255,0.05)' }}>
            <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
              <div style={{ background: 'var(--accent-violet)', color: '#fff', width: 20, height: 20, borderRadius: '50%', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '0.75rem', fontWeight: 'bold' }}>3</div>
              <div style={{ fontWeight: 600, fontSize: '0.85rem', color: 'var(--text-primary)' }}>Browse & Discover</div>
            </div>
            <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)', lineHeight: 1.5 }}>
              Open Burp's built-in browser and surf your target. Traffic is automatically sent here, and vulnerabilities will instantly appear in Burp's <strong>Issues</strong> panel!
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="glass-card mb-6" style={{
      borderLeft: '3px solid var(--accent-cyan)',
      background: 'linear-gradient(135deg, rgba(0, 212, 255, 0.05) 0%, transparent 100%)'
    }}>
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Activity size={18} color="var(--accent-cyan)" />
          <h3 style={{ fontFamily: 'var(--font-display)', fontWeight: 700, fontSize: '0.95rem' }}>Active Burp Suite Bridge</h3>
        </div>
        <div className="badge badge-blue flex items-center gap-1">
          <RefreshCw size={12} className="animate-spin" /> Live Sync
        </div>
      </div>

      <div className="flex flex-col gap-3">
        {sessions.map(session => (
          <div key={session.session_id} style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'space-between',
            padding: '12px 16px',
            background: 'rgba(0,0,0,0.2)',
            borderRadius: 'var(--radius-sm)',
            border: '1px solid rgba(255,255,255,0.05)'
          }}>
            <div>
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: '0.85rem', color: 'var(--text-primary)', marginBottom: 4 }}>
                {session.target_url}
              </div>
              <div style={{ fontSize: '0.75rem', color: 'var(--text-muted)' }}>
                Session ID: {session.session_id ? session.session_id.split('-')[0] : 'N/A'}...
              </div>
            </div>

            <div className="flex items-center gap-6">
              <div className="text-center">
                <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Traffic Processed</div>
                <div style={{ fontFamily: 'var(--font-mono)', fontWeight: 600, color: 'var(--text-primary)' }}>
                  {session.imported_traffic_count}
                </div>
              </div>
              <div className="text-center">
                <div style={{ fontSize: '0.7rem', color: 'var(--text-muted)', textTransform: 'uppercase' }}>Findings Synced</div>
                <div style={{ fontFamily: 'var(--font-mono)', fontWeight: 600, color: 'var(--color-warning)' }}>
                  {session.exported_findings_count}
                </div>
              </div>
              <div className={`badge ${session.status === 'Syncing' ? 'badge-warning' : 'badge-success'}`}>
                {session.status}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
