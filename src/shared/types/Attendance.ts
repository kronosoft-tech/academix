export type AttendanceStatus = "present" | "absent" | "late" | "excused";

export interface Attendance {
  id: string;
  studentId: string;
  groupId: string;
  date: string;
  status: AttendanceStatus;
  notes?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateAttendanceInput {
  studentId: string;
  groupId: string;
  date: string;
  status: AttendanceStatus;
  notes?: string;
}

export interface BulkAttendanceInput {
  groupId: string;
  date: string;
  records: {
    studentId: string;
    status: AttendanceStatus;
    notes?: string;
  }[];
}

export interface UpdateAttendanceInput {
  status?: AttendanceStatus;
  notes?: string;
}
