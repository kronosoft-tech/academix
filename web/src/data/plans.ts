export interface Plan {
  id: 'basico' | 'pro' | 'premium';
  name: string;
  priceUSD: number;
  priceCOP: number;
  maxStudents: number | null;
  maxUsers: number | null;
  features: string[];
}

export const PLANS: Plan[] = [
  {
    id: 'basico',
    name: 'Básico',
    priceUSD: 29,
    priceCOP: 89900,
    maxStudents: 100,
    maxUsers: 1,
    features: [
      'Hasta 100 estudiantes',
      '1 usuario administrador',
      'Gestión de cursos y grupos',
      'Control de asistencia',
      'Reportes básicos (PDF)',
      'Soporte por email',
    ],
  },
  {
    id: 'pro',
    name: 'Pro',
    priceUSD: 49,
    priceCOP: 149900,
    maxStudents: 500,
    maxUsers: 5,
    features: [
      'Hasta 500 estudiantes',
      '5 usuarios con roles',
      'Cobros y facturación automática',
      'Reportes avanzados y métricas',
      'Asistente IA para gestión',
      'Notificaciones automáticas',
      'Soporte prioritario',
    ],
  },
  {
    id: 'premium',
    name: 'Premium',
    priceUSD: 79,
    priceCOP: 259900,
    maxStudents: null,
    maxUsers: null,
    features: [
      'Estudiantes ilimitados',
      'Usuarios ilimitados',
      'Multi-sede',
      'Reportes personalizados',
      'Asistente IA avanzado',
      'Acceso API',
      'Soporte dedicado 24/7',
      'Onboarding personalizado',
    ],
  },
];
