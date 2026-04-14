'use client';

import { useEffect, useState } from 'react';
import { useRouter, usePathname } from 'next/navigation';
import { isAuthenticated } from '@/lib/auth';
import Sidebar from '@/components/Sidebar';

export default function AppLayoutWrapper({ children }: { children: React.ReactNode }) {
  const router = useRouter();
  const pathname = usePathname();
  const [authState, setAuthState] = useState<'checking' | 'authenticated' | 'unauthenticated'>('checking');

  // Consider any path starting with /auth as public
  const isAuthPage = pathname?.startsWith('/auth');

  useEffect(() => {
    const authenticated = isAuthenticated();

    if (authenticated) {
      if (isAuthPage) {
        // Authenticated user on auth page -> redirect to dashboard
        router.replace('/');
      } else {
        setAuthState('authenticated');
      }
    } else {
      if (!isAuthPage) {
        // Unauthenticated user on protected route -> redirect to login
        router.replace('/auth/login');
      }
      setAuthState('unauthenticated');
    }
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
    </div>
  );
}

