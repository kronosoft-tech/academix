export interface Student {
  id: string;
  userId: string;
  name: string;
  firstName?: string;
  lastName?: string;
  documentNumber: string;
  documentType: "cc" | "ti" | "ce" | "rc" | "nip";
  email: string;
  phone?: string;
  address?: string;
  birthDate?: string;
  guardianName?: string;
  guardianDocument?: string;
  guardianPhone?: string;
  courseId?: string;
  groupId?: string;
  courseName?: string;
  groupName?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreateStudentInput {
  name: string;
  documentNumber: string;
  documentType: Student["documentType"];
  email: string;
  phone?: string;
  address?: string;
  birthDate?: string;
  guardianName?: string;
  guardianDocument?: string;
  guardianPhone?: string;
  courseId?: string;
  groupId?: string;
}

export interface UpdateStudentInput {
  name?: string;
  documentNumber?: string;
  documentType?: Student["documentType"];
  email?: string;
  phone?: string;
  address?: string;
  birthDate?: string;
  guardianName?: string;
  guardianDocument?: string;
  guardianPhone?: string;
  courseId?: string;
  groupId?: string;
}
