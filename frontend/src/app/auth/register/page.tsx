'use client';

import { useState, useEffect, useMemo } from 'react';
import { registerUser } from '@/lib/auth';
import { useRouter } from 'next/navigation';
import { Shield, Mail, Lock, User, ArrowRight, Eye, EyeOff, Loader2, Check, X } from 'lucide-react';
import Image from 'next/image';
import Link from 'next/link';

function getPasswordStrength(password: string): { score: number; label: string; color: string } {
  let score = 0;
  if (password.length >= 6) score++;
  if (password.length >= 10) score++;
  if (/[A-Z]/.test(password)) score++;
  if (/[0-9]/.test(password)) score++;
  if (/[^A-Za-z0-9]/.test(password)) score++;

  if (score <= 1) return { score: 1, label: 'Weak', color: '#ef4444' };
  if (score <= 2) return { score: 2, label: 'Fair', color: '#f97316' };
  if (score <= 3) return { score: 3, label: 'Good', color: '#eab308' };
  if (score <= 4) return { score: 4, label: 'Strong', color: '#22c55e' };
  return { score: 5, label: 'Excellent', color: '#10b981' };
}

export default function RegisterPage() {
  const router = useRouter();
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [shake, setShake] = useState(false);
  const [mounted, setMounted] = useState(false);

  const passwordStrength = useMemo(() => getPasswordStrength(password), [password]);
  const passwordsMatch = confirmPassword.length > 0 && password === confirmPassword;
  const passwordsMismatch = confirmPassword.length > 0 && password !== confirmPassword;

  useEffect(() => {
    setMounted(true);
  }, []);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');

    if (password !== confirmPassword) {
      setError('Passwords do not match');
      setShake(true);
      setTimeout(() => setShake(false), 600);
      return;
    }

    if (password.length < 6) {
      setError('Password must be at least 6 characters');
      setShake(true);
      setTimeout(() => setShake(false), 600);
      return;
    }

    setLoading(true);

    try {
      await registerUser(name, email, password);
      setSuccess(true);
      setTimeout(() => {
        router.replace('/');
      }, 1200);
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : 'Registration failed';
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
          <h1 className="auth-title">Create Account</h1>
          <p className="auth-description">Join the security intelligence platform</p>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="auth-form">
          {/* Name field */}
          <div className={`auth-input-group ${name ? 'has-value' : ''}`}>
            <div className="auth-input-icon">
              <User size={18} />
            </div>
            <input
              id="register-name"
              type="text"
              className="auth-input"
              placeholder=" "
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
              autoComplete="name"
            />
            <label htmlFor="register-name" className="auth-input-label">Full Name</label>
            <div className="auth-input-glow" />
          </div>

          {/* Email field */}
          <div className={`auth-input-group ${email ? 'has-value' : ''}`}>
            <div className="auth-input-icon">
              <Mail size={18} />
            </div>
            <input
              id="register-email"
              type="email"
              className="auth-input"
              placeholder=" "
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              required
              autoComplete="email"
            />
            <label htmlFor="register-email" className="auth-input-label">Email Address</label>
            <div className="auth-input-glow" />
          </div>

          {/* Password field */}
          <div className={`auth-input-group ${password ? 'has-value' : ''}`}>
            <div className="auth-input-icon">
              <Lock size={18} />
            </div>
            <input
              id="register-password"
              type={showPassword ? 'text' : 'password'}
              className="auth-input"
              placeholder=" "
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              autoComplete="new-password"
              minLength={6}
            />
            <label htmlFor="register-password" className="auth-input-label">Password</label>
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

          {/* Password strength indicator */}
          <div 
            className="password-strength" 
            style={{ 
              opacity: password.length > 0 ? 1 : 0, 
              pointerEvents: 'none',
              transition: 'opacity 0.3s ease'
            }}
          >
            <div className="password-strength-bar-track">
              <div
                className="password-strength-bar-fill"
                style={{
                  width: password.length > 0 ? `${(passwordStrength.score / 5) * 100}%` : '0%',
                  background: passwordStrength.color,
                  boxShadow: password.length > 0 ? `0 0 12px ${passwordStrength.color}40` : 'none',
                }}
              />
            </div>
            <span className="password-strength-label" style={{ color: passwordStrength.color }}>
              {password.length > 0 ? passwordStrength.label : ''}
            </span>
          </div>

          {/* Confirm password field */}
          <div className={`auth-input-group ${confirmPassword ? 'has-value' : ''} ${passwordsMatch ? 'input-match' : ''} ${passwordsMismatch ? 'input-mismatch' : ''}`}>
            <div className="auth-input-icon">
              <Lock size={18} />
            </div>
            <input
              id="register-confirm"
              type={showConfirm ? 'text' : 'password'}
              className="auth-input"
              placeholder=" "
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              required
              autoComplete="new-password"
              minLength={6}
            />
            <label htmlFor="register-confirm" className="auth-input-label">Confirm Password</label>
            <div className="auth-input-status">
              {passwordsMatch && <Check size={16} className="match-icon" />}
              {passwordsMismatch && <X size={16} className="mismatch-icon" />}
            </div>
            <button
              type="button"
              className="auth-input-toggle"
              onClick={() => setShowConfirm(!showConfirm)}
              tabIndex={-1}
              style={{ right: confirmPassword.length > 0 ? '36px' : '14px' }}
            >
              {showConfirm ? <EyeOff size={16} /> : <Eye size={16} />}
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
                Creating account...
              </>
            ) : success ? (
              <>
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                  <polyline points="20 6 9 17 4 12" className="auth-checkmark" />
                </svg>
                Account Created!
              </>
            ) : (
              <>
                Create Account
                <ArrowRight size={18} />
              </>
            )}
          </button>
        </form>

        {/* Footer */}
        <div className="auth-footer">
          <span className="auth-footer-text">Already have an account?</span>
          <Link href="/auth/login" className="auth-footer-link">
            Sign In
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
