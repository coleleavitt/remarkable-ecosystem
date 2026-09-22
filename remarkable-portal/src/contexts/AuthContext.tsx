import { createContext, useContext, useState, useEffect, useCallback, type ReactNode } from 'react';
import { auth as authApi } from '../api/client';
import type { User, LoginCredentials } from '../types';

interface AuthContextType {
  user: User | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (credentials: LoginCredentials) => Promise<void>;
  logout: () => void;
  refreshUser: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<User | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  
  const refreshUser = useCallback(async () => {
    if (authApi.isAuthenticated()) {
      try {
        const userData = await authApi.getUser();
        setUser(userData);
      } catch {
        setUser(null);
        authApi.logout();
      }
    }
  }, []);
  
  useEffect(() => {
    refreshUser().finally(() => setIsLoading(false));
  }, [refreshUser]);
  
  const login = async (credentials: LoginCredentials) => {
    await authApi.login(credentials);
    await refreshUser();
  };
  
  const logout = () => {
    authApi.logout();
    setUser(null);
  };
  
  return (
    <AuthContext.Provider value={{
      user,
      isAuthenticated: !!user,
      isLoading,
      login,
      logout,
      refreshUser,
    }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth must be used within AuthProvider');
  }
  return context;
}
