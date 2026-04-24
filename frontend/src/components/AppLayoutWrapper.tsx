'use client';

import { useEffect, useState } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import { isAuthenticated, verifyToken, logout } from '@/lib/auth';
import Sidebar from '@/components/Sidebar';
import SessionSidebar from '@/components/SessionSidebar';

export default function AppLayoutWrapper({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();
  const [authState, setAuthState] = useState<'checking' | 'authenticated' | 'unauthenticated'>('checking');

  // Consider any path starting with /auth as public
  const isAuthPage = pathname?.startsWith('/auth');

  useEffect(() => {
    let cancelled = false;

    async function validateSession() {
      // Fast check: no token at all → skip API call
      if (!isAuthenticated()) {
        if (!isAuthPage) {
          router.replace('/auth/login');
        }
        if (!cancelled) setAuthState('unauthenticated');
        return;
      }

      // Token exists → validate against backend /auth/me
      const user = await verifyToken();

      if (cancelled) return;

      if (user) {
        // Token is valid — backend confirmed
        if (isAuthPage) {
          router.replace('/');
        } else {
          setAuthState('authenticated');
        }
      } else {
        // Token was invalid/expired — verifyToken already cleared localStorage
        logout();
        if (!isAuthPage) {
          router.replace('/auth/login');
        }
        setAuthState('unauthenticated');
      }
    }

    validateSession();
    return () => { cancelled = true; };
  }, [pathname, isAuthPage, router]);

  // Auth pages: always render immediately (no guard needed)
  if (isAuthPage) {
    return <>{children}</>;
  }

  // Protected pages: show spinner while checking
  if (authState !== 'authenticated') {
    return (
      <div style={{
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        minHeight: '100vh',
        background: '#030711',
      }}>
        <div className="auth-spinner" style={{
          width: '40px',
          height: '40px',
          border: '3px solid rgba(0, 212, 255, 0.1)',
          borderTopColor: '#00d4ff',
          borderRadius: '50%',
        }} />
      </div>
    );
  }

  // Normal protected app layout
  return (
    <div className="app-layout">
      <Sidebar />
      <main className="main-content">
        {children}
      </main>
      <SessionSidebar />
    </div>
  );
}
