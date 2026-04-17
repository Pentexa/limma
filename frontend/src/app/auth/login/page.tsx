'use client';

import { useState, useEffect } from 'react';
import { loginUser } from '@/lib/auth';
import { useRouter } from 'next/navigation';
import { Shield, Mail, Lock, ArrowRight, Eye, EyeOff, Loader2 } from 'lucide-react';
import Image from 'next/image';
import Link from 'next/link';

export default function LoginPage() {
  const router = useRouter();
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [shake, setShake] = useState(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      await loginUser(email, password);
      setSuccess(true);
      setTimeout(() => {
        router.replace('/');
      }, 1200);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Login failed';
      setError(message);
      setShake(true);
      setTimeout(() => setShake(false), 600);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="auth-page">
      {/* Floating orbs background */}
      <div className="auth-orb auth-orb-1" />
      <div className="auth-orb auth-orb-2" />
      <div className="auth-orb auth-orb-3" />
      <div className="auth-orb auth-orb-4" />
      <div className="auth-orb auth-orb-5" />

      {/* Grid overlay */}
      <div className="auth-grid-overlay" />

      <div className={`auth-card ${mounted ? 'auth-card-visible' : ''} ${shake ? 'auth-card-shake' : ''} ${success ? 'auth-card-success' : ''}`}>
        {/* Logo */}
        <div className="auth-logo">
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', marginRight: '6px' }}>
            <Image src="/limma_logo_icon.svg" alt="Limma Logo" width={52} height={52} style={{ filter: 'drop-shadow(0 4px 16px rgba(0, 212, 255, 0.4))' }} />
          </div>
          <div className="auth-logo-text-group">
            <span className="auth-logo-text">LIMMA</span>
            <span className="auth-logo-subtitle">Security Intelligence Platform</span>
          </div>
        </div>

        {/* Header */}
        <div className="auth-header">
          <h1 className="auth-title">Welcome Back</h1>
          <p className="auth-description">Sign in to access your security dashboard</p>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="auth-form">
          <div className={`auth-input-group ${email ? 'has-value' : ''}`}>
            <div className="auth-input-icon">
              <Mail size={18} />
            </div>
            <input
              id="login-email"
              type="email"
              className="auth-input"
              placeholder=" "
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
              autoComplete="email"
            />
            <label htmlFor="login-email" className="auth-input-label">Email Address</label>
            <div className="auth-input-glow" />
          </div>

          <div className={`auth-input-group ${password ? 'has-value' : ''}`}>
            <div className="auth-input-icon">
              <Lock size={18} />
            </div>
            <input
              id="login-password"
              type={showPassword ? 'text' : 'password'}
              className="auth-input"
              placeholder=" "
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              autoComplete="current-password"
              minLength={6}
            />
            <label htmlFor="login-password" className="auth-input-label">Password</label>
            <button
              type="button"
              className="auth-input-toggle"
              onClick={() => setShowPassword(!showPassword)}
              tabIndex={-1}
            >
              {showPassword ? <EyeOff size={16} /> : <Eye size={16} />}
            </button>
            <div className="auth-input-glow" />
          </div>

          {error && (
            <div className="auth-error">
              <span className="auth-error-icon">!</span>
              {error}
            </div>
          )}

          <button
            type="submit"
            className={`auth-button ${loading ? 'auth-button-loading' : ''} ${success ? 'auth-button-success' : ''}`}
            disabled={loading || success}
          >
            {loading ? (
              <>
                <Loader2 size={18} className="auth-spinner" />
                Signing in...
              </>
            ) : success ? (
              <>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="20 6 9 17 4 12" className="auth-checkmark" />
                </svg>
                Success!
              </>
            ) : (
              <>
                Sign In
                <ArrowRight size={18} />
              </>
            )}
          </button>
        </form>

        {/* Footer */}
        <div className="auth-footer">
          <span className="auth-footer-text">Don&apos;t have an account?</span>
          <Link href="/auth/register" className="auth-footer-link">
            Create Account
            <ArrowRight size={14} />
          </Link>
        </div>

        {/* Decorative corner accents */}
        <div className="auth-corner auth-corner-tl" />
        <div className="auth-corner auth-corner-tr" />
        <div className="auth-corner auth-corner-bl" />
        <div className="auth-corner auth-corner-br" />
      </div>
    </div>
  );
}
