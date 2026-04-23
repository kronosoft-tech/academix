import type { ReactNode } from "react";
import { cn } from "../../../lib/utils";

interface Column<T> {
  key: keyof T | string;
  header: string;
  render?: (item: T) => ReactNode;
  className?: string;
}

interface TableProps<T> {
  data: T[];
  columns: Column<T>[];
  emptyMessage?: string;
  className?: string;
}

export function Table<T extends object>({
  data,
  columns,
  emptyMessage = "No data available",
  className,
}: TableProps<T>) {
  if (data.length === 0) {
    return (
      <div className={cn("text-center py-8 text-[var(--color-foreground)]/60", className)}>
        {emptyMessage}
      </div>
    );
  }

  return (
    <div className={cn("overflow-x-auto", className)}>
      <table className="min-w-full divide-y divide-[var(--color-foreground)]/20">
        <thead className="bg-[var(--color-secondary)]/5">
          <tr>
            {columns.map((column) => (
              <th
                key={String(column.key)}
                className={cn(
                  "px-6 py-3 text-left text-xs font-medium text-[var(--color-foreground)]/60 uppercase tracking-wider",
                  column.className
                )}
              >
                {column.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="bg-[var(--color-background)] divide-y divide-[var(--color-foreground)]/10">
          {data.map((item, index) => (
            <tr key={index} className="hover:bg-[var(--color-secondary)]/5">
              {columns.map((column) => (
                <td
                  key={String(column.key)}
                  className={cn("px-6 py-4 whitespace-nowrap text-sm text-[var(--color-foreground)]", column.className)}
                >
                  {column.render
                    ? column.render(item)
                    : String(item[column.key as keyof T] ?? "")}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}