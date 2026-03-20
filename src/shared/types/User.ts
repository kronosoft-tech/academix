export type Role = "admin" | "gerente" | "empleado" | "profesor";

export interface User {
  id: string;
  email: string;
  name: string;
  role: Role;
  createdAt: string;
  updatedAt: string;
}

export interface CreateUserInput {
  email: string;
  password: string;
  name: string;
  role: Role;
}

export interface UpdateUserInput {
  email?: string;
  name?: string;
  role?: Role;
}
