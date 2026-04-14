'use client';

import { useState, useRef } from 'react';
import { Globe, Zap, Loader2, ArrowRight } from 'lucide-react';

interface UrlInputProps {
  onSubmit: (url: string) => void;
  loading?: boolean;
  placeholder?: string;
  buttonLabel?: string;
}

function normalizeUrl(input: string): string {
  let u = input.trim();
  if (!u) return u;
  // If no protocol, prepend https://
  if (!/^https?:\/\//i.test(u)) {
    u = 'https://' + u;
  }
  return u;
}

export default function UrlInput({ onSubmit, loading, placeholder, buttonLabel }: UrlInputProps) {
  const [url, setUrl] = useState('');
  const [focused, setFocused] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  const normalized = normalizeUrl(url);
  const showHint = url.trim().length > 0 && !/^https?:\/\//i.test(url.trim());

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (normalized && !loading) {
      onSubmit(normalized);
    }
  };

  return (
    <form className="url-input-wrapper" onSubmit={handleSubmit} style={{ position: 'relative' }}>
      {/* Ambient glow when focused */}
      {focused && (
        <div style={{
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          width: '120%',
          height: '300%',
          background: 'radial-gradient(ellipse, rgba(0, 212, 255, 0.04) 0%, transparent 70%)',
          pointerEvents: 'none',
          zIndex: 0,
          transition: 'opacity 0.3s',
        }} />
      )}
      <div className="url-input-container" style={{ position: 'relative', zIndex: 1 }}>
        <Globe size={18} />
        <input
          ref={inputRef}
          type="text"
          className="url-input"
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onFocus={() => setFocused(true)}
          onBlur={() => setFocused(false)}
          placeholder={placeholder || 'https://example.com'}
          disabled={loading}
        />
      </div>
      <button
        type="submit"
        className={`scan-button ${loading ? 'scanning' : ''}`}
        disabled={!normalized || loading}
        style={{ position: 'relative', zIndex: 1 }}
      >
        {loading ? (
          <>
            <Loader2 size={16} style={{ animation: 'spin 1s linear infinite' }} />
            <span>Scanning...</span>
          </>
        ) : (
          <>
            <Zap size={16} />
            <span>{buttonLabel || 'Scan'}</span>
            <ArrowRight size={14} style={{ marginLeft: 2, opacity: 0.6 }} />
          </>
        )}
      </button>
      {showHint && (
        <div style={{
          position: 'absolute',
          bottom: -24,
          left: 0,
          fontSize: '0.72rem',
          color: 'var(--accent-cyan)',
          fontFamily: 'var(--font-mono)',
          display: 'flex',
          alignItems: 'center',
          gap: 4,
          zIndex: 1,
          opacity: 0.7,
        }}>
          <span style={{ color: 'var(--text-muted)' }}>→</span> Will scan as: <span style={{ color: 'var(--accent-blue)' }}>{normalized}</span>
        </div>
      )}
    </form>
  );
}
