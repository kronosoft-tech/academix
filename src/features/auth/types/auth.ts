export interface LoginCredentials {
  email: string;
  password: string;
}

export interface AuthResponse {
  user: {
    id: string;
    email: string;
    name: string;
    role: "admin" | "teacher" | "student";
  };
  token: string;
}
