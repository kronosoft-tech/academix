export interface Group {
  id: string;
  courseId: string;
  name: string;
  professorId?: string;
  schedule?: string;
  days?: string[];
  startTime?: string;
  endTime?: string;
  startDate?: string;
  endDate?: string;
  maxStudents: number;
  currentStudents?: number;
  status: "open" | "closed" | "completed";
  createdAt: string;
  updatedAt: string;
  // Include course info for price lookup
  coursePrice?: number;
  courseName?: string;
  // New fields for class duration and skipped dates
  classDuration?: number;
  skippedDates?: string[];
  calculatedEndDate?: string;
}

export interface CreateGroupInput {
  courseId: string;
  name: string;
  professorId?: string;
  schedule?: string;
  days?: string[];
  startTime?: string;
  endTime?: string;
  startDate?: string;
  endDate?: string;
  maxStudents: number;
  classDuration?: number;
  skippedDates?: string[];
}

export interface UpdateGroupInput {
  name?: string;
  schedule?: string;
  days?: string[];
  startTime?: string;
  endTime?: string;
  startDate?: string;
  endDate?: string;
  maxStudents?: number;
  professorId?: string;
  status?: Group["status"];
  classDuration?: number;
  skippedDates?: string[];
}
