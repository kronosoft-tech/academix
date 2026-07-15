import { Badge } from "../../../shared/ui/components/Badge";

interface AttendanceWarningBadgeProps {
  absenceCount: number;
  threshold: number;
  showCount?: boolean;
}

export function AttendanceWarningBadge({
  absenceCount,
  threshold,
  showCount = true,
}: AttendanceWarningBadgeProps) {
  if (absenceCount <= threshold) return null;

  return (
    <Badge variant="danger">
      {showCount
        ? `Más de ${threshold} faltas`
        : "Advertencia"}
    </Badge>
  );
}
