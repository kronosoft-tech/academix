# Academix 🎓

Sistema de gestión académica para academias de enseñanza. Aplicación de escritorio construida con Tauri 2, React 19 y TypeScript.

## 🚀 Características

### Módulos del Sistema

| Módulo | Descripción |
|--------|-------------|
| **Dashboard** | Vista principal con métricas totales de estudiantes, cursos, grupos y pagos pendientes |
| **Estudiantes** | CRUD completo de estudiantes con información de contacto, acudiente y estado de inscripción |
| **Cursos** | Gestión de cursos con precio, descripción y duración |
| **Grupos** | Creación de grupos asociados a cursos con horarios (días, hora inicio/fin) y fechas (inicio/fin) |
| **Asistencia** | Registro daily de asistencia por grupo con estadísticas de asistencia |
| **Pagos** | Sistema de seguimiento de pagos con estado (paid/pending/overdue), monto, fecha de vencimiento |
| **Usuarios** | Administración de usuarios del sistema |
| **Autenticación** | Login seguro con credenciales |

### Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Tauri 2 (Rust)
- **Base de Datos**: SQLite
- **Estado**: Zustand 5
- **Estilos**: Tailwind CSS 4
- **UI**: Componentes personalizados con Lucide Icons

### Arquitectura

El proyecto sigue una arquitectura hexagonal en el backend:

```
src-tauri/
├── domain/           # Entidades y lógica de dominio
├── application/      # Casos de uso, DTOs y puertos
├── infrastructure/   # Repositorios y base de datos
└── commands/         # Comandos Tauri
```

## 📋 Requisitos Previos

- [Bun](https://bun.sh/) (gestor de paquetes)
- [Rust](https://www.rust-lang.org/) (última versión estable)
- [Node.js](https://nodejs.org/) 18+

## 🛠️ Instalación

```bash
# 1. Instalar dependencias
bun install

# 2. Iniciar desarrollo (frontend + backend)
bun run tauri dev
```

## 📦 Scripts Disponibles

| Comando | Descripción |
|---------|-------------|
| `bun run dev` | Iniciar servidor de desarrollo |
| `bun run build` | Compilar aplicación para producción |
| `bun run tauri dev` | Desarrollo completo Tauri |
| `bun run tauri build` | Build de producción Tauri |
| `bun run test` | Ejecutar tests unitarios |
| `bun run test:e2e` | Ejecutar tests E2E con Playwright |
| `bunx tsc --noEmit` | Verificar tipos TypeScript |

## 🔐 Credenciales por Defecto

El sistema incluye un usuario administrador por defecto:

- **Usuario**: `admin`
- **Contraseña**: `admin123`

## 📁 Estructura del Proyecto

```
academix/
├── src/                    # Frontend (React 19)
│   ├── app/               # Layout, router, componentes globales
│   ├── features/          # Módulos por funcionalidad
│   │   ├── auth/          # Autenticación
│   │   ├── dashboard/    # Dashboard principal
│   │   ├── students/      # Gestión de estudiantes
│   │   ├── courses/      # Gestión de cursos
│   │   ├── groups/       # Gestión de grupos
│   │   ├── attendance/   # Registro de asistencia
│   │   ├── payments/     # Seguimiento de pagos
│   │   └── users/        # Administración de usuarios
│   ├── shared/           # Componentes, hooks, tipos compartidos
│   │   ├── components/   # UI components
│   │   ├── hooks/        # Hooks reutilizables
│   │   ├── types/        # Tipos TypeScript
│   │   └── utils/        # Utilidades
│   └── lib/              # Configuración y utilidades
├── src-tauri/             # Backend (Rust)
│   ├── migrations/       # Migraciones SQL
│   ├── src/
│   │   ├── domain/       # Entidades de dominio
│   │   ├── application/  # Capa de aplicación
│   │   ├── infrastructure/# Capa de infraestructura
│   │   └── commands/     # Comandos Tauri
│   └── tauri.conf.json   # Configuración Tauri
└── package.json
```

## 🖥️ Capturas de Pantalla

### Dashboard
Muestra estadísticas en tiempo real de estudiantes, cursos, grupos y pagos pendientes con acciones rápidas.

### Estudiantes
Lista paginada con búsqueda, formulario completo para datos del estudiante y acudiente.

### Grupos
Visualización de grupos por curso con registro daily de asistencia y estadísticas.

### Pagos
Tabla de pagos con filtros por estado, modal de detalles, seguimiento de vencimiento.

## 🤝 Contribuir

1. Fork del repositorio
2. Crear branch (`git checkout -b feature/nueva-caracteristica`)
3. Commit con mensajes convencionales
4. Push y crear Pull Request

## 📄 Licencia

MIT License
