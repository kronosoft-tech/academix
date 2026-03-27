import { useState, useMemo } from "react";
import type { PaymentStatusSummary } from "../../../shared/types/Payment";
import { Input } from "../../../shared/ui/components/Input";

interface PaymentsTableProps {
  paymentSummaries: PaymentStatusSummary[];
  isLoading: boolean;
  onStudentClick: (studentId: string) => void;
  searchTerm: string;
  onSearchChange: (term: string) => void;
}

type SortField = "studentId" | "studentName" | "groupName" | "status" | "daysDelayed";
type SortDirection = "asc" | "desc";

export function PaymentsTable({
  paymentSummaries,
  isLoading,
  onStudentClick,
  searchTerm,
  onSearchChange,
}: PaymentsTableProps) {
  const [sortField, setSortField] = useState<SortField>("status");
  const [sortDirection, setSortDirection] = useState<SortDirection>("asc");

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(sortDirection === "asc" ? "desc" : "asc");
    } else {
      setSortField(field);
      setSortDirection("asc");
    }
  };

  const getSortIndicator = (field: SortField) => {
    if (sortField !== field) return null;
    return sortDirection === "asc" ? " ↑" : " ↓";
  };

  const statusOrder = { delinquent: 0, current: 1, ahead: 2 };

  const filteredAndSortedSummaries = useMemo(() => {
    const filtered = paymentSummaries.filter(
      (summary) =>
        summary.studentName.toLowerCase().includes(searchTerm.toLowerCase()) ||
        summary.studentId.toLowerCase().includes(searchTerm.toLowerCase()) ||
        summary.groupName.toLowerCase().includes(searchTerm.toLowerCase())
    );

    return filtered.sort((a, b) => {
      let comparison = 0;
      switch (sortField) {
        case "studentId":
          comparison = a.studentId.localeCompare(b.studentId);
          break;
        case "studentName":
          comparison = a.studentName.localeCompare(b.studentName);
          break;
        case "groupName":
          comparison = a.groupName.localeCompare(b.groupName);
          break;
        case "status":
          comparison = statusOrder[a.status as keyof typeof statusOrder] - statusOrder[b.status as keyof typeof statusOrder];
          break;
        case "daysDelayed":
          comparison = a.daysDelayed - b.daysDelayed;
          break;
      }
      return sortDirection === "asc" ? comparison : -comparison;
    });
  }, [paymentSummaries, searchTerm, sortField, sortDirection]);

  const getStatusBadge = (status: string) => {
    const statusConfig = {
      current: {
        class: "bg-green-100 text-green-800",
        label: "Al día",
      },
      delinquent: {
        class: "bg-red-100 text-red-800",
        label: "Atrasado",
      },
      ahead: {
        class: "bg-blue-100 text-blue-800",
        label: "Adelantado",
      },
    };
    const config = statusConfig[status as keyof typeof statusConfig] || statusConfig.current;
    return (
      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${config.class}`}>
        {config.label}
      </span>
    );
  };

  const getDaysDelayed = (daysDelayed: number) => {
    if (daysDelayed < 0) {
      return <span className="text-blue-600">-{Math.abs(daysDelayed)} días</span>;
    } else if (daysDelayed === 0) {
      return <span className="text-green-600">Hoy</span>;
    } else {
      return <span className="text-red-600">+{daysDelayed} días</span>;
    }
  };

  if (isLoading) {
    return (
      <div className="text-center py-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto"></div>
        <p className="mt-2 text-gray-500">Cargando...</p>
      </div>
    );
  }

  if (filteredAndSortedSummaries.length === 0) {
    return (
      <div className="text-center py-12">
        <p className="text-gray-500">
          {searchTerm ? "No se encontraron resultados" : "No hay estudiantes con pagos registrados"}
        </p>
      </div>
    );
  }

  return (
    <div>
      <div className="mb-4">
        <Input
          placeholder="Buscar por estudiante, ID o grupo..."
          value={searchTerm}
          onChange={(e) => onSearchChange(e.target.value)}
        />
      </div>

      <div className="overflow-x-auto">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100" onClick={() => handleSort("studentId")}>
                ID Estudiante{getSortIndicator("studentId")}
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100" onClick={() => handleSort("studentName")}>
                Nombre{getSortIndicator("studentName")}
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100" onClick={() => handleSort("groupName")}>
                Grupo{getSortIndicator("groupName")}
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100" onClick={() => handleSort("status")}>
                Estado{getSortIndicator("status")}
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100" onClick={() => handleSort("daysDelayed")}>
                Días de atraso{getSortIndicator("daysDelayed")}
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Fecha Vencimiento
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                Acciones
              </th>
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {filteredAndSortedSummaries.map((summary) => (
              <tr
                key={summary.studentId}
                className="hover:bg-gray-50"
              >
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {summary.studentId.substring(0, 8)}...
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                  {summary.studentName}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                  {summary.groupName}
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  {getStatusBadge(summary.status)}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm">
                  {getDaysDelayed(summary.daysDelayed)}
                </td>
                <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                  {new Date(summary.dueDate).toLocaleDateString("es-CO")}
                </td>
                <td className="px-6 py-4 whitespace-nowrap">
                  <button
                    onClick={() => onStudentClick(summary.studentId)}
                    className="text-blue-600 hover:text-blue-800 text-sm font-medium"
                  >
                    Ver Detalles
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}