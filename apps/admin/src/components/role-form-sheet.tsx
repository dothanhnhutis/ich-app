import { useEffect, useState } from "react";
import * as z from "zod";
import { useForm } from "@tanstack/react-form";

import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Field, FieldLabel } from "@/components/ui/field";
import { Skeleton } from "@/components/ui/skeleton";
import {
  api,
  ApiError,
  type GroupedPermissions,
  type RoleResponse,
} from "@/lib/api";

const schema = z.object({
  name: z
    .string()
    .min(1, "Tên vai trò 1-255 ký tự")
    .max(255, "Tên vai trò 1-255 ký tự"),
  description: z.string().max(1000, "Mô tả tối đa 1000 ký tự"),
  status: z.enum(["ACTIVE", "DEACTIVATED"]),
});

function fieldErrors(errors: unknown[]): string {
  return errors
    .map((e) =>
      typeof e === "string"
        ? e
        : ((e as { message?: string } | null)?.message ?? ""),
    )
    .filter(Boolean)
    .join(", ");
}

export function RoleFormSheet({
  open,
  onOpenChange,
  role,
  onSaved,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  role?: RoleResponse | null;
  onSaved: () => void;
}) {
  const isEdit = !!role;
  const [catalog, setCatalog] = useState<GroupedPermissions | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [formError, setFormError] = useState<string | null>(null);
  const [permError, setPermError] = useState<string | null>(null);

  const form = useForm({
    defaultValues: {
      name: role?.name ?? "",
      description: role?.description ?? "",
      status: (role?.status === "DEACTIVATED" ? "DEACTIVATED" : "ACTIVE") as
        | "ACTIVE"
        | "DEACTIVATED",
    },
    validators: { onSubmit: schema },
    onSubmit: async ({ value }) => {
      setFormError(null);
      if (selected.size === 0) {
        setPermError("Phải chọn ít nhất một quyền");
        return;
      }
      const permission_ids = Array.from(selected);
      try {
        if (isEdit && role) {
          await api.roles.update(role.id, {
            name: value.name,
            description: value.description,
            status: value.status,
            permission_ids,
          });
        } else {
          await api.roles.create({
            name: value.name,
            description: value.description,
            permission_ids,
          });
        }
        onSaved();
        onOpenChange(false);
      } catch (err) {
        setFormError(
          err instanceof ApiError
            ? err.message
            : "Lưu thất bại. Vui lòng thử lại.",
        );
      }
    },
  });

  // Mỗi lần mở (hoặc đổi role): reset form + nạp danh mục quyền + tick sẵn quyền hiện có.
  useEffect(() => {
    if (!open) return;
    let active = true;

    form.reset({
      name: role?.name ?? "",
      description: role?.description ?? "",
      status: role?.status === "DEACTIVATED" ? "DEACTIVATED" : "ACTIVE",
    });
    setFormError(null);
    setPermError(null);
    setSelected(new Set());

    setCatalog(null);
    api.permissions
      .list()
      .then((c) => active && setCatalog(c))
      .catch(() => active && setCatalog({ groups: [] }));

    if (role) {
      api.roles
        .permissionsOf(role.id)
        .then((g) => {
          if (!active) return;
          setSelected(
            new Set(g.groups.flatMap((grp) => grp.permissions.map((p) => p.id))),
          );
        })
        .catch(() => {});
    }

    return () => {
      active = false;
    };
    // form là instance ổn định; chỉ chạy lại khi mở/đổi role.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open, role?.id]);

  const togglePerm = (id: string, checked: boolean) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (checked) next.add(id);
      else next.delete(id);
      return next;
    });
    setPermError(null);
  };

  return (
    <Sheet open={open} onOpenChange={(o) => onOpenChange(o)}>
      <SheetContent className="w-full sm:max-w-md">
        <SheetHeader>
          <SheetTitle>{isEdit ? "Sửa vai trò" : "Tạo vai trò"}</SheetTitle>
          <SheetDescription>
            {isEdit
              ? "Cập nhật thông tin và quyền của vai trò."
              : "Nhập thông tin và chọn quyền cho vai trò mới."}
          </SheetDescription>
        </SheetHeader>

        <form
          onSubmit={(e) => {
            e.preventDefault();
            form.handleSubmit();
          }}
          className="flex min-h-0 flex-1 flex-col"
        >
          <div className="flex-1 space-y-4 overflow-y-auto px-6 pb-4">
            {formError && (
              <p role="alert" className="text-sm text-destructive">
                {formError}
              </p>
            )}

            <form.Field
              name="name"
              children={(field) => {
                const err = fieldErrors(field.state.meta.errors);
                return (
                  <Field>
                    <FieldLabel htmlFor={field.name}>Tên vai trò</FieldLabel>
                    <Input
                      id={field.name}
                      name={field.name}
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(e) => field.handleChange(e.target.value)}
                      aria-invalid={!!err}
                      placeholder="VD: Quản lý kho"
                    />
                    {err && <p className="text-xs text-destructive">{err}</p>}
                  </Field>
                );
              }}
            />

            <form.Field
              name="description"
              children={(field) => {
                const err = fieldErrors(field.state.meta.errors);
                return (
                  <Field>
                    <FieldLabel htmlFor={field.name}>Mô tả</FieldLabel>
                    <textarea
                      id={field.name}
                      name={field.name}
                      rows={3}
                      value={field.state.value}
                      onBlur={field.handleBlur}
                      onChange={(e) => field.handleChange(e.target.value)}
                      className="rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/30"
                      placeholder="Mô tả ngắn (tuỳ chọn)"
                    />
                    {err && <p className="text-xs text-destructive">{err}</p>}
                  </Field>
                );
              }}
            />

            {isEdit && (
              <form.Field
                name="status"
                children={(field) => (
                  <Field>
                    <FieldLabel htmlFor={field.name}>Trạng thái</FieldLabel>
                    <select
                      id={field.name}
                      name={field.name}
                      value={field.state.value}
                      onChange={(e) =>
                        field.handleChange(
                          e.target.value as "ACTIVE" | "DEACTIVATED",
                        )
                      }
                      className="h-9 rounded-md border border-input bg-transparent px-3 text-sm shadow-xs outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/30"
                    >
                      <option value="ACTIVE">Hoạt động</option>
                      <option value="DEACTIVATED">Vô hiệu</option>
                    </select>
                  </Field>
                )}
              />
            )}

            <Field>
              <FieldLabel>Quyền ({selected.size} đã chọn)</FieldLabel>
              <div className="max-h-[38vh] space-y-4 overflow-y-auto rounded-md border p-3">
                {catalog === null ? (
                  <div className="space-y-2">
                    {Array.from({ length: 4 }).map((_, i) => (
                      <Skeleton key={i} className="h-5 w-full" />
                    ))}
                  </div>
                ) : catalog.groups.length === 0 ? (
                  <p className="text-sm text-muted-foreground">
                    Không có quyền nào.
                  </p>
                ) : (
                  catalog.groups.map((g) => (
                    <div key={g.prefix}>
                      <div className="mb-1 text-xs font-semibold tracking-wide text-muted-foreground">
                        {g.prefix}
                      </div>
                      <div className="space-y-1">
                        {g.permissions.map((p) => (
                          <label
                            key={p.id}
                            className="flex cursor-pointer items-start gap-2 text-sm"
                          >
                            <input
                              type="checkbox"
                              className="mt-0.5 size-4 accent-primary"
                              checked={selected.has(p.id)}
                              onChange={(e) =>
                                togglePerm(p.id, e.target.checked)
                              }
                            />
                            <span>
                              <span className="font-medium">{p.code}</span>
                              {p.description && (
                                <span className="text-muted-foreground">
                                  {" "}
                                  — {p.description}
                                </span>
                              )}
                            </span>
                          </label>
                        ))}
                      </div>
                    </div>
                  ))
                )}
              </div>
              {permError && (
                <p className="text-xs text-destructive">{permError}</p>
              )}
            </Field>
          </div>

          <SheetFooter className="flex-row justify-end gap-2 border-t">
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Hủy
            </Button>
            <form.Subscribe
              selector={(s) => s.isSubmitting}
              children={(isSubmitting) => (
                <Button type="submit" disabled={isSubmitting}>
                  {isSubmitting ? "Đang lưu..." : "Lưu"}
                </Button>
              )}
            />
          </SheetFooter>
        </form>
      </SheetContent>
    </Sheet>
  );
}
