const API_BASE = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8900';

export interface AuthUser {
  id: string;
  name: string;
  email: string;
}

export interface AuthResponse {
  token: string;
  user: AuthUser;
}

export async function registerUser(name: string, email: string, password: string): Promise<AuthResponse> {
  const res = await fetch(`${API_BASE}/auth/register`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, email, password }),
  });

  if (!res.ok) {
    const data = await res.json().catch(() => null);
    throw new Error(data?.error?.message || 'Registration failed');
  }

  const data: AuthResponse = await res.json();
  localStorage.setItem('limma_token', data.token);
  localStorage.setItem('limma_user', JSON.stringify(data.user));
  return data;
}

export async function loginUser(email: string, password: string): Promise<AuthResponse> {
  const res = await fetch(`${API_BASE}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ email, password }),
  });

  if (!res.ok) {
    const data = await res.json().catch(() => null);
    throw new Error(data?.error?.message || 'Login failed');
  }

  const data: AuthResponse = await res.json();
  localStorage.setItem('limma_token', data.token);
  localStorage.setItem('limma_user', JSON.stringify(data.user));
  return data;
}

export function getToken(): string | null {
  if (typeof window === 'undefined') return null;
  return localStorage.getItem('limma_token');
}

export function getUser(): AuthUser | null {
  if (typeof window === 'undefined') return null;
  const raw = localStorage.getItem('limma_user');
  if (!raw) return null;
  try {
    return JSON.parse(raw);
  } catch {
    return null;
  }
}

export function isAuthenticated(): boolean {
  return !!getToken();
}

export function logout(): void {
  localStorage.removeItem('limma_token');
  localStorage.removeItem('limma_user');
}

/**
 * Verify the stored JWT against the backend `/auth/me` endpoint.
 * Returns fresh user data if the token is valid, null if expired/invalid.
 * On failure, automatically clears stale credentials from localStorage.
 */
export async function verifyToken(): Promise<AuthUser | null> {
  const token = getToken();
  if (!token) return null;

  try {
    const res = await fetch(`${API_BASE}/auth/me`, {
      method: 'GET',
      headers: {
        'Authorization': `Bearer ${token}`,
        'Content-Type': 'application/json',
      },
    });

    if (!res.ok) {
      // Token is expired or invalid — clear stale credentials
      logout();
      return null;
    }

    const user: AuthUser = await res.json();
    // Sync localStorage with fresh backend data
    localStorage.setItem('limma_user', JSON.stringify(user));
    return user;
  } catch {
    // Network error — don't clear credentials (might be temporary)
    return null;
  }
}

/**
 * Convenience wrapper: verify token and update local user state.
 * Returns true if the session is valid, false otherwise.
 */
export async function refreshUser(): Promise<boolean> {
  const user = await verifyToken();
  return user !== null;
}
