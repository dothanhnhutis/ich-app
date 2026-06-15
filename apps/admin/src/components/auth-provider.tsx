import { AuthContext, type AuthState } from "@/contexts/auth-context";
import { api } from "@/lib/api";
import {
  useCallback,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";

export function AuthProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<AuthState>({
    profile: null,
    hydrating: true,
  });

  // Hydrate phiên khi tải/refresh trang: hỏi /users/me qua cookie httpOnly.
  useEffect(() => {
    let active = true;
    api
      .me()
      .then((profile) => {
        if (active) setState({ profile, hydrating: false });
      })
      .catch(() => {
        if (active) setState({ profile: null, hydrating: false });
      });
    return () => {
      active = false;
    };
  }, []);

  const login = useCallback(async (email: string, password: string) => {
    const { profile } = await api.login(email, password);
    setState({ profile, hydrating: false });
  }, []);

  const logout = useCallback(async () => {
    try {
      await api.logout();
    } finally {
      setState({ profile: null, hydrating: false });
    }
  }, []);

  const value = useMemo(
    () => ({ ...state, login, logout }),
    [state, login, logout],
  );

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}
