import { useEffect, useState } from "react";
import { UsersIcon, CalendarIcon, TrendingUpIcon } from "lucide-react";
import { useAttendance, GroupAttendanceStats } from "../hooks/useAttendance";

interface Props {
  groupId: string;
  totalStudents: number;
}

interface StatBarProps {
  label: string;
  count: number;
  percentage: number;
  color: string;
  bgColor: string;
}

function StatBar({ label, count, percentage, color, bgColor }: StatBarProps) {
  return (
    <div className="mb-4">
      <div className="flex justify-between items-center mb-1">
        <span className="text-sm font-medium text-gray-700">{label}</span>
        <span className="text-sm font-bold" style={{ color }}>
          {percentage.toFixed(1)}%
        </span>
      </div>
      <div className={`w-full h-3 rounded-full ${bgColor}`}>
        <div
          className={`h-3 rounded-full transition-all duration-500 ${color.replace("text-", "bg-")}`}
          style={{ width: `${Math.max(percentage, 0)}%` }}
        />
      </div>
      <span className="text-xs text-gray-500 mt-0.5">
        {count} registros
      </span>
    </div>
  );
}

export default function GroupAttendanceStatsView({ groupId, totalStudents }: Props) {
  const { getGroupStats } = useAttendance();
  const [stats, setStats] = useState<GroupAttendanceStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadStats = async () => {
      setLoading(true);
      setError(null);
      const { stats: fetchedStats, error: fetchError } = await getGroupStats(
        groupId,
        totalStudents
      );
      if (fetchError) {
        setError(fetchError);
      } else {
        setStats(fetchedStats);
      }
      setLoading(false);
    };

    loadStats();
  }, [groupId, totalStudents, getGroupStats]);

  if (loading) {
    return (
      <div className="flex items-center justify-center py-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4 text-red-700">
        Error al cargar estadísticas: {error}
      </div>
    );
  }

  if (!stats) {
    return (
      <div className="bg-gray-50 border border-gray-200 rounded-lg p-4 text-gray-600">
        No hay datos de asistencia para este grupo.
      </div>
    );
  }

  if (stats.totalRecords === 0) {
    return (
      <div className="bg-blue-50 border border-blue-200 rounded-lg p-6">
        <div className="flex items-center gap-3 mb-2">
          <CalendarIcon className="w-6 h-6 text-blue-600" />
          <h3 className="text-lg font-semibold text-blue-900">Control de Asistencia Grupal</h3>
        </div>
        <p className="text-blue-700">
          No hay registros de asistencia aún. Comienza a pasar lista para ver las estadísticas.
        </p>
        <div className="mt-4 grid grid-cols-3 gap-4">
          <div className="bg-white rounded-lg p-4 text-center">
            <p className="text-2xl font-bold text-gray-400">0</p>
            <p className="text-sm text-gray-500">Estudiantes</p>
          </div>
          <div className="bg-white rounded-lg p-4 text-center">
            <p className="text-2xl font-bold text-gray-400">0</p>
            <p className="text-sm text-gray-500">Sesiones</p>
          </div>
          <div className="bg-white rounded-lg p-4 text-center">
            <p className="text-2xl font-bold text-gray-400">0</p>
            <p className="text-sm text-gray-500">Registros</p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          <TrendingUpIcon className="w-6 h-6 text-blue-600" />
          <h3 className="text-lg font-semibold">Estadísticas de Asistencia Grupal</h3>
        </div>
      </div>

      {/* Summary cards */}
      <div className="grid grid-cols-3 gap-4 mb-6">
        <div className="bg-gradient-to-br from-blue-50 to-blue-100 rounded-lg p-4 border border-blue-200">
          <div className="flex items-center gap-2 mb-1">
            <UsersIcon className="w-4 h-4 text-blue-600" />
            <p className="text-sm text-blue-600 font-medium">Estudiantes</p>
          </div>
          <p className="text-2xl font-bold text-blue-900">{stats.totalStudents}</p>
        </div>
        <div className="bg-gradient-to-br from-purple-50 to-purple-100 rounded-lg p-4 border border-purple-200">
          <div className="flex items-center gap-2 mb-1">
            <CalendarIcon className="w-4 h-4 text-purple-600" />
            <p className="text-sm text-purple-600 font-medium">Sesiones</p>
          </div>
          <p className="text-2xl font-bold text-purple-900">{stats.totalSessions}</p>
        </div>
        <div className="bg-gradient-to-br from-gray-50 to-gray-100 rounded-lg p-4 border border-gray-200">
          <div className="flex items-center gap-2 mb-1">
            <TrendingUpIcon className="w-4 h-4 text-gray-600" />
            <p className="text-sm text-gray-600 font-medium">Total Registros</p>
          </div>
          <p className="text-2xl font-bold text-gray-900">{stats.totalRecords}</p>
        </div>
      </div>

      {/* Attendance percentage bars */}
      <div className="bg-gray-50 rounded-lg p-4 mb-4">
        <h4 className="text-sm font-semibold text-gray-700 mb-4">Distribución de Asistencia</h4>
        
        <StatBar
          label="Presente"
          count={stats.presentCount}
          percentage={stats.presentPercentage}
          color="text-green-600"
          bgColor="bg-gray-200"
        />
        
        <StatBar
          label="Ausente"
          count={stats.absentCount}
          percentage={stats.absentPercentage}
          color="text-red-600"
          bgColor="bg-gray-200"
        />
        
        <StatBar
          label="Tarde"
          count={stats.lateCount}
          percentage={stats.latePercentage}
          color="text-yellow-600"
          bgColor="bg-gray-200"
        />
        
        <StatBar
          label="Justificado"
          count={stats.excusedCount}
          percentage={stats.excusedPercentage}
          color="text-blue-600"
          bgColor="bg-gray-200"
        />
      </div>

      {/* Quick summary */}
      <div className="flex items-center justify-between text-sm text-gray-600 pt-4 border-t">
        <span>
          Asistencia efectiva (Presente + Tarde):{" "}
          <strong className="text-green-600">
            {(stats.presentPercentage + stats.latePercentage).toFixed(1)}%
          </strong>
        </span>
        <span>
          Promedio por estudiante:{" "}
          <strong>
            {stats.totalStudents > 0
              ? (stats.totalRecords / stats.totalStudents).toFixed(1)
              : 0}{" "}
            registros
          </strong>
        </span>
      </div>
    </div>
  );
}
