import { createFileRoute } from "@tanstack/react-router";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";

const PAGE_SIZES = [10, 20, 50, 100];

export const Route = createFileRoute("/_protected/roles")({
  validateSearch: (search: Record<string, unknown>) => {
    const page = Math.max(1, Math.trunc(Number(search.page)) || 1);
    const rawSize = Number(search.page_size);
    const page_size = PAGE_SIZES.includes(rawSize) ? rawSize : 20;
    return { page, page_size };
  },
  component: RouteComponent,
});

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  MoreHorizontalIcon,
  PencilIcon,
  PlusIcon,
  Trash2Icon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { StatusBadge } from "@/components/StatusBadge";
import { useState } from "react";
import {
  keepPreviousData,
  useQuery,
  useQueryClient,
} from "@tanstack/react-query";
import TablePagenation from "@/components/table-pagenation";
import { RoleFormSheet } from "@/components/role-form-sheet";
import { DeleteRoleDialog } from "@/components/delete-role-dialog";
import { api, type RoleResponse } from "@/lib/api";

function RouteComponent() {
  const { page, page_size } = Route.useSearch();
  const navigate = Route.useNavigate();
  const queryClient = useQueryClient();

  const { data, isPending } = useQuery({
    queryKey: ["roles", { page, page_size }],
    queryFn: () => api.roles.list({ page, page_size }),
    placeholderData: keepPreviousData,
  });

  const invalidate = () =>
    queryClient.invalidateQueries({ queryKey: ["roles"] });

  const [creating, setCreating] = useState(false);
  const [editing, setEditing] = useState<RoleResponse | null>(null);
  const [deleting, setDeleting] = useState<RoleResponse | null>(null);

  const canUpdate = true; //useHasPermission("ROLE_UPDATE");
  const canDelete = true; //useHasPermission("ROLE_DELETE");

  return (
    <div className="container mx-auto px-4 pb-10 pt-4">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-lg font-semibold">Vai trò</h1>
        <Button onClick={() => setCreating(true)}>
          <PlusIcon />
          Tạo vai trò
        </Button>
      </div>
      <div className="overflow-x-auto rounded-md border bg-card">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Tên</TableHead>
              <TableHead>Mô tả</TableHead>
              <TableHead>Trạng thái</TableHead>
              <TableHead className="w-15"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {isPending || !data ? (
              Array.from({ length: 5 }).map((_, i) => (
                <TableRow key={i}>
                  {Array.from({ length: 5 }).map((_, j) => (
                    <TableCell key={j}>
                      <Skeleton className="h-4 w-full" />
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : data.items.length === 0 ? (
              <TableRow>
                <TableCell
                  colSpan={4}
                  className="h-24 text-center text-muted-foreground"
                >
                  {false ? "Không có kết quả phù hợp." : "Chưa có vai trò nào."}
                </TableCell>
              </TableRow>
            ) : (
              data.items.map((r) => {
                const showUpdate = canUpdate && r.can_update;
                const showDelete = canDelete && r.can_delete;
                return (
                  <TableRow key={r.id}>
                    <TableCell className="font-medium">{r.name}</TableCell>
                    <TableCell className="max-w-xs truncate text-muted-foreground">
                      {r.description || "—"}
                    </TableCell>

                    <TableCell>
                      <StatusBadge status={r.status} />
                    </TableCell>
                    <TableCell>
                      {(showUpdate || showDelete) && (
                        <DropdownMenu>
                          <DropdownMenuTrigger
                            render={
                              <Button
                                variant="ghost"
                                size="icon"
                                className="h-8 w-8"
                              />
                            }
                          >
                            <MoreHorizontalIcon className="h-4 w-4" />
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            {showUpdate && (
                              <DropdownMenuItem onClick={() => setEditing(r)}>
                                <PencilIcon className="mr-2 h-4 w-4" />
                                Sửa
                              </DropdownMenuItem>
                            )}
                            {showDelete && (
                              <DropdownMenuItem
                                className="text-destructive focus:text-destructive"
                                onClick={() => setDeleting(r)}
                              >
                                <Trash2Icon className="mr-2 h-4 w-4" />
                                Xóa
                              </DropdownMenuItem>
                            )}
                          </DropdownMenuContent>
                        </DropdownMenu>
                      )}
                    </TableCell>
                  </TableRow>
                );
              })
            )}
          </TableBody>
        </Table>
      </div>
      <div className="mt-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex items-center gap-3 text-sm text-muted-foreground">
          <span>Tổng {data?.total_items ?? 0} vai trò</span>
          <label className="flex items-center gap-2">
            <span>Mỗi trang</span>
            <select
              value={page_size}
              onChange={(e) =>
                navigate({
                  search: (prev) => ({
                    ...prev,
                    page_size: Number(e.target.value),
                    page: 1,
                  }),
                })
              }
              className="h-8 rounded-md border border-input bg-transparent px-2 text-sm outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/30"
            >
              {PAGE_SIZES.map((s) => (
                <option key={s} value={s}>
                  {s}
                </option>
              ))}
            </select>
          </label>
        </div>
        <TablePagenation
          currPage={page}
          totalPage={data?.total_pages ?? 1}
          hasNextPage={data?.has_next ?? false}
          onPageChange={(p) =>
            navigate({ search: (prev) => ({ ...prev, page: p }) })
          }
        />
      </div>

      <RoleFormSheet
        open={creating || editing !== null}
        role={editing}
        onOpenChange={(o) => {
          if (!o) {
            setCreating(false);
            setEditing(null);
          }
        }}
        onSaved={invalidate}
      />
      <DeleteRoleDialog
        role={deleting}
        onOpenChange={(o) => {
          if (!o) setDeleting(null);
        }}
        onDeleted={invalidate}
      />
    </div>
  );
}
