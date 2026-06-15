export type UserStatus = "ACTIVE" | "DEACTIVATED" | "PENDING_PASSWORD";

export type ProfileResponse = {
  id: string;
  email: string;
  username: string | null;
  status: UserStatus;
  created_at: string;
  updated_at: string;
};

// ----- Roles & permissions -----

export type PaginatedResponse<T> = {
  items: T[];
  page: number;
  page_size: number;
  total_items: number;
  total_pages: number;
  has_next: boolean;
  has_prev: boolean;
};

export type RoleResponse = {
  id: string;
  name: string;
  description: string;
  status: string;
  can_delete: boolean;
  can_update: boolean;
  created_at: string;
  updated_at: string;
};

export type CreateRoleBody = {
  name: string;
  description?: string;
  permission_ids: string[];
};

export type UpdateRoleBody = {
  name?: string;
  description?: string;
  status?: string;
  permission_ids?: string[];
};

export type PermissionResponse = {
  id: string;
  code: string;
  description: string;
};

export type PermissionGroup = {
  prefix: string;
  permissions: PermissionResponse[];
};

export type GroupedPermissions = {
  groups: PermissionGroup[];
};

// Base URL của API.
// - Dev: để trống → gọi đường dẫn tương đối "/api/..." đi qua Vite proxy
//   (same-origin nên cookie phiên `session` hoạt động, không cần CORS).
// - Production: set VITE_API_URL để gọi trực tiếp tới backend.
const API_URL = import.meta.env.VITE_API_URL ?? "";

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, {
    ...init,
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
      ...init?.headers,
    },
  });

  const isJson = res.headers.get("content-type")?.includes("application/json");
  const body = isJson ? await res.json().catch(() => null) : null;

  if (!res.ok) {
    const message =
      body && typeof body.error === "string"
        ? body.error
        : "Có lỗi xảy ra. Vui lòng thử lại.";
    throw new ApiError(res.status, message);
  }

  return body as T;
}

type LoginResponse = {
  user_id: string;
  session: string;
  expires_in: number;
};

export const api = {
  // Đăng nhập: backend set cookie httpOnly `session`; sau đó lấy profile qua /me.
  async login(
    email: string,
    password: string,
  ): Promise<{ profile: ProfileResponse }> {
    await request<LoginResponse>("/api/v1/auth/login", {
      method: "POST",
      body: JSON.stringify({ email, password, device_type: "web" }),
    });
    const profile = await api.me();
    return { profile };
  },

  // Lấy người dùng hiện tại từ cookie phiên (401 nếu chưa/đã hết phiên).
  me(): Promise<ProfileResponse> {
    return request<ProfileResponse>("/api/v1/users/me");
  },

  // Đăng xuất phiên hiện tại (backend xoá cookie).
  async logout(): Promise<void> {
    await request<unknown>("/api/v1/auth/logout", { method: "POST" });
  },

  roles: {
    list(
      params: {
        page?: number;
        page_size?: number;
        name?: string;
        status?: string;
      } = {},
    ): Promise<PaginatedResponse<RoleResponse>> {
      const qs = new URLSearchParams();
      if (params.page != null) qs.set("page", String(params.page));
      if (params.page_size != null)
        qs.set("page_size", String(params.page_size));
      if (params.name) qs.set("name", params.name);
      if (params.status) qs.set("status", params.status);
      const q = qs.toString();
      return request<PaginatedResponse<RoleResponse>>(
        `/api/v1/roles${q ? `?${q}` : ""}`,
      );
    },

    create(body: CreateRoleBody): Promise<RoleResponse> {
      return request<RoleResponse>("/api/v1/roles", {
        method: "POST",
        body: JSON.stringify(body),
      });
    },

    update(id: string, body: UpdateRoleBody): Promise<RoleResponse> {
      return request<RoleResponse>(`/api/v1/roles/${id}`, {
        method: "PATCH",
        body: JSON.stringify(body),
      });
    },

    remove(id: string): Promise<{ message: string }> {
      return request<{ message: string }>(`/api/v1/roles/${id}`, {
        method: "DELETE",
      });
    },

    // Quyền hiện có của 1 role (gom theo nhóm prefix).
    permissionsOf(id: string): Promise<GroupedPermissions> {
      return request<GroupedPermissions>(`/api/v1/roles/${id}/permissions`);
    },
  },

  permissions: {
    // Toàn bộ danh mục quyền (gom theo nhóm prefix).
    list(): Promise<GroupedPermissions> {
      return request<GroupedPermissions>("/api/v1/permissions");
    },
  },
};
