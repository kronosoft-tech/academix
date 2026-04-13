export interface Course {
  id: string;
  name: string;
  description?: string;
  code: string;
  price: number;
  duration: number; // in hours
  createdAt: string;
  updatedAt: string;
}

export interface CreateCourseInput {
  name: string;
  description?: string;
  code: string;
  price: number;
  duration: number;
}

export interface UpdateCourseInput {
  name?: string;
  description?: string;
  code?: string;
  price?: number;
  duration?: number;
}
