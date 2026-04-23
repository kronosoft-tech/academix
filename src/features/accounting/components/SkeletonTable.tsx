// SkeletonTable Component - Phase 7
// Skeleton loader for tables

import { cn } from "../../../lib/utils";

interface SkeletonTableProps {
  rows?: number;
  columns: number;
  className?: string;
}

export function SkeletonTable({
  rows = 5,
  columns,
  className,
}: SkeletonTableProps) {
  // Generate column widths for variety
  const columnWidths = Array.from({ length: columns }, (_, i) => {
    const widths = ["w-16", "w-24", "w-32", "w-40", "w-48", "w-64", "flex-1"];
    return widths[i % widths.length];
  });

  return (
    <div className={cn("w-full overflow-hidden rounded-lg border border-[var(--color-foreground)]/20", className)}>
      <table className="w-full">
        <thead className="bg-[var(--color-foreground)]/5">
          <tr>
            {Array.from({ length: columns }).map((_, i) => (
              <th key={i} className="h-10 px-4 text-left">
                <div className="h-4 w-20 animate-pulse rounded bg-slate-200" />
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-100">
          {Array.from({ length: rows }).map((_, rowIndex) => (
            <tr key={rowIndex} className="hover:bg-[var(--color-foreground)]/5/50">
              {Array.from({ length: columns }).map((_, colIndex) => (
                <td key={colIndex} className="h-12 px-4">
                  <div
                    className={cn(
                      "h-4 animate-pulse rounded bg-slate-200",
                      columnWidths[colIndex]
                    )}
                  />
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// SkeletonCard for dashboard stats
interface SkeletonCardProps {
  className?: string;
}

export function SkeletonCard({ className }: SkeletonCardProps) {
  return (
    <div
      className={cn(
        "rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6",
        className
      )}
    >
      <div className="mb-2 h-4 w-24 animate-pulse rounded bg-slate-200" />
      <div className="h-8 w-32 animate-pulse rounded bg-slate-200" />
    </div>
  );
}

// SkeletonChart for dashboard charts
interface SkeletonChartProps {
  className?: string;
}

export function SkeletonChart({ className }: SkeletonChartProps) {
  return (
    <div
      className={cn(
        "rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6",
        className
      )}
    >
      <div className="mb-4 h-5 w-40 animate-pulse rounded bg-slate-200" />
      <div className="flex h-48 items-end justify-around gap-2">
        {Array.from({ length: 6 }).map((_, i) => (
          <div
            key={i}
            className="w-full animate-pulse rounded-t bg-slate-200"
            style={{ height: `${30 + Math.random() * 50}%` }}
          />
        ))}
      </div>
    </div>
  );
}

// SkeletonForm for loading forms
interface SkeletonFormProps {
  fields?: number;
  className?: string;
}

export function SkeletonForm({ fields = 4, className }: SkeletonFormProps) {
  return (
    <div className={cn("space-y-4 rounded-lg border border-[var(--color-foreground)]/20 bg-[var(--color-background)] p-6", className)}>
      {Array.from({ length: fields }).map((_, i) => (
        <div key={i}>
          <div className="mb-1 h-3 w-20 animate-pulse rounded bg-slate-200" />
          <div className="h-10 animate-pulse rounded border border-[var(--color-foreground)]/20 bg-[var(--color-foreground)]/5" />
        </div>
      ))}
      <div className="mt-6 h-10 w-24 animate-pulse rounded bg-slate-200" />
    </div>
  );
}