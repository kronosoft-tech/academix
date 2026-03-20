export interface Group {
  id: string;
  courseId: string;
  name: string;
  professorId?: string;
  schedule?: string;
  days?: string[];
  startTime?: string;
  endTime?: string;
  maxStudents: number;
  currentStudents?: number;
  status: "open" | "closed" | "completed";
  createdAt: string;
  updatedAt: string;
}

export interface CreateGroupInput {
  courseId: string;
  name: string;
  professorId?: string;
  schedule?: string;
  days?: string[];
  startTime?: string;
  endTime?: string;
  maxStudents: number;
}

export interface UpdateGroupInput {
  name?: string;
  schedule?: string;
  days?: string[];
  startTime?: string;
  endTime?: string;
  maxStudents?: number;
  professorId?: string;
  status?: Group["status"];
}
