import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface User {
  id: string;
  username: string;
  email: string;
  balance: number;
  avatar: string;
}

interface AuthState {
  token: string | null;
  user: User | null;
  loading: boolean;
  error: string | null;
  setToken: (token: string | null) => void;
  setUser: (user: User | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  login: (username: string, password: string) => Promise<void>;
  register: (username: string, email: string, password: string) => Promise<void>;
  loadUser: () => Promise<void>;
  logout: () => void;
}

const API = 'http://localhost:3001/api';

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      token: null,
      user: null,
      loading: false,
      error: null,

      setToken: (token) => set({ token }),
      setUser: (user) => set({ user }),
      setLoading: (loading) => set({ loading }),
      setError: (error) => set({ error }),

      login: async (username, password) => {
        set({ loading: true, error: null });
        try {
          const res = await fetch(`${API}/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password }),
          });
          const data = await res.json();
          if (!res.ok) throw new Error(data.error || 'Erro ao fazer login');
          set({ token: data.token, user: data.user, loading: false });
        } catch (err: any) {
          set({ error: err.message, loading: false });
          throw err;
        }
      },

      register: async (username, email, password) => {
        set({ loading: true, error: null });
        try {
          const res = await fetch(`${API}/auth/register`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, email, password }),
          });
          const data = await res.json();
          if (!res.ok) throw new Error(data.error || 'Erro ao registrar');
          set({ token: data.token, user: data.user, loading: false });
        } catch (err: any) {
          set({ error: err.message, loading: false });
          throw err;
        }
      },

      loadUser: async () => {
        const { token } = get();
        if (!token) return;
        set({ loading: true });
        try {
          const res = await fetch(`${API}/auth/me`, {
            headers: { Authorization: `Bearer ${token}` },
          });
          if (!res.ok) throw new Error('Sessão expirada');
          const data = await res.json();
          set({ user: data.user, loading: false });
        } catch {
          set({ token: null, user: null, loading: false });
        }
      },

      logout: () => {
        set({ token: null, user: null, error: null });
      },
    }),
    {
      name: 'poker-auth',
      partialize: (state) => ({ token: state.token }),
    }
  )
);