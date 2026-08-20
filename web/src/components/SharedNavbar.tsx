import { useState, useEffect } from 'react';

interface UserData {
  authenticated: boolean;
  name?: string;
  email?: string;
  role?: string;
}

const marketingLinks = [
  { label: 'Producto', href: '#features' },
  { label: 'Precios', href: '/pricing' },
  { label: 'Descargar', href: '/downloads' },
  { label: 'Tutoriales', href: '/tutorials' },
  { label: 'Blog', href: '/blog' },
  { label: 'Contacto', href: '/contact' },
];

const dashboardLinks = [
  { label: 'Resumen', href: '/dashboard' },
  { label: 'Suscripción', href: '/dashboard/subscription' },
  { label: 'Pagos', href: '/dashboard/payments' },
  { label: 'Soporte', href: '/dashboard/support' },
];

function getInitials(name: string): string {
  return name
    .split(' ')
    .map((w) => w[0])
    .slice(0, 2)
    .join('')
    .toUpperCase();
}

function isDashboardRoute(pathname: string): boolean {
  return pathname.startsWith('/dashboard');
}

export default function SharedNavbar() {
  const [mobileOpen, setMobileOpen] = useState(false);
  const [user, setUser] = useState<UserData | null>(null);
  const [pathname, setPathname] = useState('/');

  useEffect(() => {
    setPathname(window.location.pathname);
    fetch('/api/me')
      .then((res) => res.json())
      .then((data) => setUser(data))
      .catch(() => setUser(null));
  }, []);

  const isDashboard = isDashboardRoute(pathname);
  const links = isDashboard ? dashboardLinks : marketingLinks;

  return (
    <>
      <header className="fixed inset-x-0 top-0 z-50 border-b border-slate-800 bg-slate-950/90 backdrop-blur">
        <nav className="mx-auto flex h-[72px] max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
          <a href="/" className="text-xl font-extrabold tracking-tight text-emerald-500">
            Academix
          </a>

          <div className="hidden items-center gap-1 md:flex">
            {links.map(({ label, href }) => (
              <a
                key={label}
                href={href}
                className="rounded-lg px-3 py-2 text-sm font-medium text-slate-300 transition-colors hover:bg-slate-800/60 hover:text-white"
              >
                {label}
              </a>
            ))}
          </div>

          <div className="hidden items-center gap-3 md:flex">
            {user?.authenticated ? (
              <details className="group relative">
                <summary className="flex cursor-pointer list-none select-none items-center gap-2 rounded-lg px-2 py-1 transition-colors hover:bg-slate-800/60 [&::-webkit-details-marker]:hidden">
                  <span className="flex h-9 w-9 items-center justify-center rounded-full bg-emerald-600 text-sm font-semibold text-white">
                    {getInitials(user.name || 'U')}
                  </span>
                  <span className="text-sm font-medium text-slate-300">{user.name}</span>
                  <svg
                    className="h-4 w-4 text-slate-400 transition-transform group-open:rotate-180"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                  </svg>
                </summary>
                <div className="absolute right-0 top-full z-50 mt-2 w-56 rounded-lg border border-slate-700 bg-slate-800 py-1 shadow-xl">
                  <div className="border-b border-slate-700 px-4 py-2">
                    <p className="truncate text-sm font-medium text-white">{user.name}</p>
                    <p className="truncate text-xs text-slate-400">{user.email}</p>
                  </div>
                  <nav className="py-1">
                    <a
                      href="/dashboard"
                      className="flex items-center gap-2 px-4 py-2 text-sm text-slate-300 transition-colors hover:bg-slate-700 hover:text-white"
                    >
                      Dashboard
                    </a>
                    <a
                      href="/dashboard"
                      className="flex items-center gap-2 px-4 py-2 text-sm text-slate-300 transition-colors hover:bg-slate-700 hover:text-white"
                    >
                      Mi Academia
                    </a>
                    <a
                      href="/dashboard/subscription"
                      className="flex items-center gap-2 px-4 py-2 text-sm text-slate-300 transition-colors hover:bg-slate-700 hover:text-white"
                    >
                      Suscripción
                    </a>
                    <a
                      href="/downloads"
                      className="flex items-center gap-2 px-4 py-2 text-sm text-slate-300 transition-colors hover:bg-slate-700 hover:text-white"
                    >
                      Descargar App
                    </a>
                  </nav>
                  <div className="border-t border-slate-700 py-1">
                    <form method="POST" action="/api/logout">
                      <button
                        type="submit"
                        className="flex w-full items-center gap-2 px-4 py-2 text-left text-sm text-red-400 transition-colors hover:bg-slate-700 hover:text-red-300"
                      >
                        Cerrar sesión
                      </button>
                    </form>
                  </div>
                </div>
              </details>
            ) : (
              <>
                <a
                  href="/auth/login"
                  className="text-sm font-medium text-slate-300 transition-colors hover:text-white"
                >
                  Iniciar sesión
                </a>
                <a
                  href="/downloads"
                  className="rounded-lg bg-emerald-600 px-3 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-500"
                >
                  Comenzar gratis
                </a>
              </>
            )}
          </div>

          <button
            type="button"
            aria-label="Abrir menú"
            className="flex h-10 w-10 items-center justify-center text-slate-100 md:hidden"
            onClick={() => setMobileOpen(true)}
          >
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
            </svg>
          </button>
        </nav>
      </header>

      {/* Mobile drawer */}
      <div
        className={`fixed inset-0 z-50 md:hidden ${mobileOpen ? 'pointer-events-auto' : 'pointer-events-none'}`}
        aria-hidden={!mobileOpen}
      >
        <div
          className={`absolute inset-0 bg-black/60 transition-opacity duration-200 ${mobileOpen ? 'opacity-100' : 'opacity-0'}`}
          onClick={() => setMobileOpen(false)}
        />
        <aside
          className={`absolute right-0 top-0 h-full w-72 border-l border-slate-800 bg-[#0f0f14] p-4 transition-transform duration-200 ${mobileOpen ? 'translate-x-0' : 'translate-x-full'}`}
        >
          <button
            type="button"
            aria-label="Cerrar menú"
            className="ml-auto flex h-10 w-10 items-center justify-center text-slate-100"
            onClick={() => setMobileOpen(false)}
          >
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
          <nav className="mt-2 flex flex-col">
            {links.map(({ label, href }) => (
              <a
                key={label}
                href={href}
                onClick={() => setMobileOpen(false)}
                className="rounded-lg px-3 py-2.5 text-sm font-medium text-slate-200 transition-colors hover:bg-slate-800"
              >
                {label}
              </a>
            ))}
          </nav>
          <div className="my-4 border-t border-slate-800" />
          <div className="flex flex-col gap-3 px-1">
            {user?.authenticated ? (
              <>
                <div className="flex items-center gap-3 rounded-lg px-3 py-2">
                  <span className="flex h-9 w-9 items-center justify-center rounded-full bg-emerald-600 text-sm font-semibold text-white">
                    {getInitials(user.name || 'U')}
                  </span>
                  <div className="min-w-0">
                    <p className="truncate text-sm font-medium text-slate-200">{user.name}</p>
                    <p className="truncate text-xs text-slate-400">{user.email}</p>
                  </div>
                </div>
                <a
                  href="/dashboard"
                  onClick={() => setMobileOpen(false)}
                  className="rounded-lg px-3 py-2.5 text-sm font-medium text-slate-200 transition-colors hover:bg-slate-800"
                >
                  Dashboard
                </a>
                <a
                  href="/dashboard/subscription"
                  onClick={() => setMobileOpen(false)}
                  className="rounded-lg px-3 py-2.5 text-sm font-medium text-slate-200 transition-colors hover:bg-slate-800"
                >
                  Suscripción
                </a>
                <a
                  href="/downloads"
                  onClick={() => setMobileOpen(false)}
                  className="rounded-lg px-3 py-2.5 text-sm font-medium text-slate-200 transition-colors hover:bg-slate-800"
                >
                  Descargar App
                </a>
                <form method="POST" action="/api/logout">
                  <button
                    type="submit"
                    className="w-full rounded-lg px-3 py-2.5 text-left text-sm font-medium text-red-400 transition-colors hover:bg-slate-800 hover:text-red-300"
                  >
                    Cerrar sesión
                  </button>
                </form>
              </>
            ) : (
              <>
                <a
                  href="/auth/login"
                  onClick={() => setMobileOpen(false)}
                  className="rounded-lg border border-slate-700 px-3 py-2 text-center text-sm font-medium text-slate-200 transition-colors hover:bg-slate-800"
                >
                  Iniciar sesión
                </a>
                <a
                  href="/downloads"
                  onClick={() => setMobileOpen(false)}
                  className="rounded-lg bg-emerald-600 px-3 py-2 text-center text-sm font-medium text-white transition-colors hover:bg-emerald-500"
                >
                  Comenzar gratis
                </a>
              </>
            )}
          </div>
        </aside>
      </div>

      {/* Spacer for the fixed header */}
      <div className="h-[72px]" />
    </>
  );
}
