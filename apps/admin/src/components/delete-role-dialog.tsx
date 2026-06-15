import { useState } from "react";

import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Button } from "@/components/ui/button";
import { api, ApiError, type RoleResponse } from "@/lib/api";

export function DeleteRoleDialog({
  role,
  onOpenChange,
  onDeleted,
}: {
  role: RoleResponse | null;
  onOpenChange: (open: boolean) => void;
  onDeleted: () => void;
}) {
  const [error, setError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  const handleDelete = async () => {
    if (!role) return;
    setError(null);
    setPending(true);
    try {
      await api.roles.remove(role.id);
      onDeleted();
      onOpenChange(false);
    } catch (err) {
      setError(
        err instanceof ApiError ? err.message : "Xoá thất bại. Vui lòng thử lại.",
      );
    } finally {
      setPending(false);
    }
  };

  return (
    <Sheet open={!!role} onOpenChange={(o) => onOpenChange(o)}>
      <SheetContent side="right" className="w-full sm:max-w-sm">
        <SheetHeader>
          <SheetTitle>Xoá vai trò</SheetTitle>
          <SheetDescription>
            Bạn có chắc muốn xoá vai trò{" "}
            <span className="font-medium text-foreground">{role?.name}</span>?
            Hành động này không thể hoàn tác.
          </SheetDescription>
        </SheetHeader>

        <div className="px-6">
          {error && <p className="text-sm text-destructive">{error}</p>}
        </div>

        <SheetFooter className="flex-row justify-end gap-2 border-t">
          <Button
            type="button"
            variant="outline"
            onClick={() => onOpenChange(false)}
          >
            Hủy
          </Button>
          <Button
            type="button"
            variant="destructive"
            disabled={pending}
            onClick={handleDelete}
          >
            {pending ? "Đang xoá..." : "Xoá"}
          </Button>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
}
