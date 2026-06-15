import type { ProfileResponse } from "@/lib/api";
import { createContext, useContext } from "react";

export type AuthState = {
  profile: ProfileResponse | null;
  hydrating: boolean;
};

export type AuthContextValue = AuthState & {
  login: (email: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
};

export const AuthContext = createContext<AuthContextValue | null>(null);

export function useAuth(): AuthContextValue {
  const ctx = useContext(AuthContext);
  if (!ctx) {
    throw new Error("useAuth phải được dùng bên trong <AuthProvider>");
  }
  return ctx;
}
