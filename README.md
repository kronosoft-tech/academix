# Academix 🎓

Sistema de gestión académica para academias de enseñanza. El repositorio contiene **dos aplicaciones**:

| App | Ubicación | Stack | Propósito |
|-----|-----------|-------|-----------|
| **Desktop** | raíz | Tauri 2 + React 19 + Rust | Gestión académica (estudiantes, cursos, grupos, asistencia, pagos, contabilidad) |
| **Web** | `web/` | Astro 7 SSR + Vercel | Sitio de marketing, dashboard de cuenta y **suscripciones/pagos** para los clientes de la app de escritorio |

Ambas usan **bun** y TypeScript, pero son paquetes independientes: nunca mezcles dependencias de la raíz con las de `web/`.

## 🚀 Características

### Módulos del Sistema (Desktop)

| Módulo | Descripción |
|--------|-------------|
| **Dashboard** | Métricas de estudiantes, cursos, grupos y pagos pendientes |
| **Estudiantes** | CRUD con información de contacto, acudiente y estado de inscripción |
| **Cursos** | Gestión con precio, descripción y duración |
| **Grupos** | Grupos asociados a cursos con horarios (días, hora inicio/fin) y fechas |
| **Asistencia** | Registro diario de asistencia por grupo con estadísticas |
| **Pagos** | Seguimiento de pagos con estado (paid/pending/overdue), monto y fecha de vencimiento |
| **Contabilidad** | Módulo de contabilidad (partidas, pasivos, activos fijos) |
| **Usuarios** | Administración de usuarios del sistema |
| **Autenticación** | Login seguro; los usuarios se registran ellos mismos (sin admin pre-cargado) |

### Web

- Páginas de marketing: inicio, precios, FAQ, contacto, descargas, tutoriales.
- Dashboard de cliente con suscripción y pagos; área de administración.
- Pasarelas de pago: **Stripe**, **Wompi** y **MercadoPago** (checkouts y webhooks).
- Cobros y recordatorios automáticos vía cron jobs; chat con IA (Groq/Cerebras); correos con Gmail.

## 🧱 Tech Stack

**Desktop**
- Frontend: React 19 + TypeScript + Vite · Zustand 5 · Tailwind CSS 4 · react-router v7 · Chart.js/Recharts · Lucide Icons
- Backend: Tauri 2 (Rust) con arquitectura hexagonal
- Base de datos: Turso (libsql) por usuario, con SQLite local como respaldo

**Web**
- Astro 7 SSR (adapter de Vercel) · React 19 · MUI + Tailwind 4
- Turso (`@libsql/client`) · Auth JWT (jose) en cookie httpOnly · nodemailer · Stripe/Wompi/MercadoPago · Groq/Cerebras

## 📋 Requisitos Previos

- [Bun](https://bun.sh/) (gestor de paquetes)
- [Rust](https://www.rust-lang.org/) (toolchain estable; necesario para Tauri)
- Linux: dependencias de WebKitGTK para compilar Tauri (`libwebkit2gtk-4.1-dev`)

## 🛠️ Instalación y Desarrollo

Ejecuta los comandos del desktop en la raíz y los de la web dentro de `web/`.

```bash
# --- Desktop (raíz) ---
bun install
bun run tauri dev      # app completa (Vite :1420 + backend Rust)
```

```bash
# --- Web (dentro de web/) ---
cd web
bun install
bun run dev            # Astro dev server en :4321
```

## 🔐 Variables de Entorno

Solo se listan los **nombres** de variables; los valores se copian de `.env.example` a un `.env` local. Nunca se commitean valores reales.

### Desktop (raíz, `.env.example`)

| Variables | Qué controlan |
|-----------|---------------|
| `ADMIN_EMAIL`, `ADMIN_PASSWORD_HASH`, `APP_IDENTIFIER` | Identidad y registro de usuarios |
| `CONTROL_PLANE_DB_URL`, `CONTROL_PLANE_DB_TOKEN`, `TURSO_API_TOKEN`, `TURSO_ORG`, `TURSO_GROUP` | Control plane de Turso → una BD por usuario |

Sin las variables de Turso la app corre **en modo degradado** (features de Turso desactivadas, respaldo SQLite local). El `.env.example` de la raíz lista las variables de identidad; el conjunto completo (incluidas las de Turso) está documentado en `AGENTS.md`.

### Web (`web/src/.env.example`)

| Variables | Qué controlan |
|-----------|---------------|
| `TURSO_URL`, `TURSO_AUTH_TOKEN` | Base de datos Turso |
| `JWT_SECRET` | Firma de sesiones |
| `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`, `STRIPE_PRICE_BASIC` | Stripe |
| `WOMPI_PUBLIC_KEY`, `WOMPI_PRIVATE_KEY`, `WOMPI_EVENTS_SECRET`, `WOMPI_API_URL` | Wompi |
| `MP_ACCESS_TOKEN`, `MP_API_URL` | MercadoPago |
| `GMAIL_USER`, `GMAIL_APP_PASSWORD` | Correo (nodemailer) |
| `GROQ_API_KEY`, `CEREBRAS_API_KEY` | Chat con IA |
| `CRON_SECRET`, `SITE_URL` | Cron jobs y URLs públicas |

Sin `TURSO_URL`/`JWT_SECRET` la web **no arranca**.

## 📦 Comandos

### Desktop (raíz)

| Comando | Descripción |
|---------|-------------|
| `bun run dev` | Vite dev server |
| `bun run build` | `tsc` + build Vite → `dist/` |
| `bun run tauri dev` | Desarrollo completo Tauri (frontend + backend) |
| `bun run tauri build` | Build de producción Tauri |
| `bunx tsc --noEmit` | Chequeo de tipos TypeScript |
| `bun run test` | Tests unitarios (Vitest) |
| `bun run test -- src/features/foo/bar.test.ts` | Un solo test |
| `bun run test:e2e` | Tests E2E (Playwright, inicia Vite automáticamente) |

### Web (dentro de `web/`)

| Comando | Descripción |
|---------|-------------|
| `bun run dev` | Astro dev server (:4321) |
| `bun run build` | `astro build` (también verifica tipos) |
| `bun run test` | Tests unitarios (Vitest) |
| `bun run test:e2e` | Tests E2E (Playwright contra :4321) |

## 🏗️ Arquitectura

El backend de escritorio sigue una **arquitectura hexagonal** y el frontend está organizado por **features**. El flujo de datos de la app desktop:

```
React → invoke("comando") → command Tauri → UseCase → Repository → SQLite local (libsql) / Turso (por usuario)
```

La web es SSR con Astro en Vercel: autenticación JWT en cookie httpOnly que separa roles `customer`/`admin`, una base Turso compartida para suscripciones, y pasarelas de pago integradas vía checkouts y webhooks.

## 📁 Estructura del Proyecto

```
academix/
├── src/               # Frontend desktop (React 19, por features)
│   ├── app/          # Layout, router, componentes globales
│   ├── features/     # auth, dashboard, students, courses, groups,
│   │                 # attendance, payments, accounting, users, updater
│   └── shared/       # Componentes, hooks y tipos compartidos
├── src-tauri/         # Backend (Rust)
│   ├── src/
│   │   ├── domain/       # Entidades y lógica de dominio
│   │   ├── application/  # Casos de uso y puertos
│   │   ├── infrastructure/# Repositorios y base de datos
│   │   └── commands/     # Comandos Tauri
│   └── migrations/       # Migraciones SQL
├── web/               # App web (Astro 7, marketing + suscripciones)
│   └── src/pages/     # Páginas de marketing, dashboard, admin y /api
├── tests/             # Tests E2E (desktop)
└── .github/workflows/ # CI/CD (build y release Tauri)
```

## 🚀 Despliegue y Releases

- **Web**: desplegada en **Vercel** (SSR). Los cron jobs de suscripciones se definen en `web/vercel.json` (expirar suscripciones 06:00, recordatorios 07:00, cobros Wompi 08:00 UTC).
- **Desktop**: `.github/workflows/tauri.yml` compila instaladores (macOS, Linux y Windows) y publica un **GitHub Release** al pushear tags `app-v*` — el release alimenta el plugin de **actualizaciones automáticas** de la app.

## 🤝 Contribuir

1. Fork del repositorio
2. Crear branch (`git checkout -b feature/nueva-caracteristica`)
3. Commit con mensajes convencionales
4. Push y crear Pull Request
