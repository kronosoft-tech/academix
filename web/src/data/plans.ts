export interface Plan {
  id: 'basico' | 'pro' | 'premium';
  name: string;
  priceCOP: number;
  maxStudents: number | null;
  maxUsers: number | null;
  features: string[];
}

export const PLANS: Plan[] = [
  {
    id: 'basico',
    name: 'Básico',
    priceCOP: 49900,
    maxStudents: 100,
    maxUsers: 1,
    features: [
      'Hasta 100 estudiantes',
      '1 usuario',
      'Reportes básicos',
      'Soporte por email',
    ],
  },
  {
    id: 'pro',
    name: 'Pro',
    priceCOP: 89900,
    maxStudents: 500,
    maxUsers: 5,
    features: [
      'Hasta 500 estudiantes',
      '5 usuarios',
      'Reportes avanzados',
      'Asistente IA',
      'Soporte prioritario',
    ],
  },
  {
    id: 'premium',
    name: 'Premium',
    priceCOP: 149900,
    maxStudents: null,
    maxUsers: null,
    features: [
      'Estudiantes ilimitados',
      'Usuarios ilimitados',
      'Reportes personalizados',
      'Asistente IA avanzado',
      'Soporte dedicado',
      'Acceso API',
    ],
  },
];
