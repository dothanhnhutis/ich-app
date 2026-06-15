import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

type PageNode = number | "…";

export default function calcPages({
  totalPage,
  currPage = 1,
  siblings = 1,
}: {
  totalPage: number;
  currPage?: number;
  siblings?: number;
}): PageNode[] {
  // Nếu tổng trang nhỏ hơn giới hạn hiển thị thì trả về liên tiếp
  const windowSize = siblings * 2 + 1;
  if (totalPage <= windowSize) {
    return Array.from({ length: totalPage }, (_, i) => i + 1);
  }

  // Hàm tạo dãy số [start..end]
  const range = (start: number, end: number) =>
    Array.from({ length: end - start + 1 }, (_, i) => start + i);

  // Tập tất cả trang cần hiển thị: 1, các trang xung quanh currPage, và totalPage
  const pages = new Set<number>([
    1,
    ...range(
      Math.max(2, currPage - siblings),
      Math.min(totalPage - 1, currPage + siblings),
    ),
    totalPage,
  ]);

  // Gom lại thành mảng và chèn "…" khi cần
  const result: PageNode[] = [];
  let prev = 0;
  for (const page of Array.from(pages).sort((a, b) => a - b)) {
    if (page - prev > 1) result.push("…");
    result.push(page);
    prev = page;
  }

  return result;
}
