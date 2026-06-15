import React from "react";
import { ChevronLeftIcon, ChevronRightIcon } from "lucide-react";

import {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
} from "./ui/pagination";
import calcPages, { cn } from "@/lib/utils";
import { Button } from "./ui/button";

const TablePagenation = (pagigation: {
  currPage: number;
  totalPage: number;
  hasNextPage: boolean;
  onPageChange: (page: number) => void;
}) => {
  const pagigations = React.useMemo(() => {
    return calcPages({
      totalPage: pagigation.totalPage,
      siblings: 1,
      currPage: pagigation.currPage,
    });
  }, [pagigation]);

  const handlePage = (page: number) => {
    pagigation.onPageChange(page);
  };

  return (
    <Pagination className="justify-end w-auto mx-0">
      <PaginationContent className="[&_a]:border">
        <PaginationItem>
          <Button
            variant={"outline"}
            disabled={pagigation.currPage == 1}
            className="gap-1 px-2.5 sm:pr-2.5"
            onClick={() => handlePage(pagigation.currPage - 1)}
          >
            <ChevronLeftIcon />
            <span className="hidden sm:block">Previous</span>
          </Button>
        </PaginationItem>
        {pagigations.map((p, idx) => {
          if (typeof p == "number") {
            return (
              <PaginationItem key={idx}>
                <Button
                  size={"icon"}
                  variant={"outline"}
                  className={cn(
                    p == pagigation.currPage
                      ? "border-primary dark:bg-accent"
                      : "",
                  )}
                  onClick={() => handlePage(p)}
                >
                  {p}
                </Button>
              </PaginationItem>
            );
          } else {
            return (
              <PaginationItem key={idx}>
                <PaginationEllipsis />
              </PaginationItem>
            );
          }
        })}
        <PaginationItem>
          <Button
            variant={"outline"}
            disabled={!pagigation.hasNextPage}
            className="gap-1 px-2.5 sm:pr-2.5"
            onClick={() => handlePage(pagigation.currPage + 1)}
          >
            <span className="hidden sm:block">Next</span>
            <ChevronRightIcon />
          </Button>
        </PaginationItem>
      </PaginationContent>
    </Pagination>
  );
};

export default TablePagenation;
