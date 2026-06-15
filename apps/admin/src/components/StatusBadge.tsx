import { Badge } from "@/components/ui/badge";

type Props = {
  status: string;
};

export function StatusBadge({ status }: Props) {
  if (status === "ACTIVE") {
    return (
      <Badge
        variant="outline"
        className="border-emerald-500/40 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400"
      >
        Hoạt động
      </Badge>
    );
  }
  if (status === "DEACTIVATED") {
    return (
      <Badge variant="secondary" className="text-muted-foreground">
        Vô hiệu
      </Badge>
    );
  }
  if (status === "PENDING_PASSWORD") {
    return (
      <Badge
        variant="outline"
        className="border-amber-500/40 bg-amber-500/10 text-amber-600 dark:text-amber-400"
      >
        Chờ đặt mật khẩu
      </Badge>
    );
  }
  return <Badge variant="outline">{status}</Badge>;
}
