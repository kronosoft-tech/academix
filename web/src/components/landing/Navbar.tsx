import { useState } from 'react';

const navLinks = [
  { label: 'Producto', href: '#features' },
  { label: 'Precios', href: '/pricing' },
  { label: 'Descargar', href: '/downloads' },
  { label: 'Tutoriales', href: '/tutorials' },
  { label: 'Contacto', href: '/contact' },
];

/**
 * Site navbar. The only interactive section of the landing page, so it is
 * the only part hydrated on the client (client:visible island in index.astro).
 * Tailwind-only: keeps MUI/emotion out of the home JS bundles.
 */
export default function Navbar() {
  const [mobileOpen, setMobileOpen] = useState(false);

  return (
    <>
      <header className="fixed inset-x-0 top-0 z-50 border-b border-slate-800 bg-slate-950/90 backdrop-blur">
        <nav className="mx-auto flex h-[72px] max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
          <a href="/" className="text-xl font-extrabold tracking-tight text-emerald-500">
            Academix
          </a>

          <div className="hidden items-center gap-1 md:flex">
            {navLinks.map(({ label, href }) => (
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
          </div>

          <button
            type="button"
            aria-label="Abrir menú"
            className="flex h-10 w-10 items-center justify-center text-slate-100 md:hidden"
            onClick={() => setMobileOpen(true)}
          >
            <svg className="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M4 6h16M4 12h16M4 18h16"
              />
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
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
          <nav className="mt-2 flex flex-col">
            {navLinks.map(({ label, href }) => (
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
            <a
              href="/auth/login"
              className="rounded-lg border border-slate-700 px-3 py-2 text-center text-sm font-medium text-slate-200 transition-colors hover:bg-slate-800"
            >
              Iniciar sesión
            </a>
            <a
              href="/downloads"
              className="rounded-lg bg-emerald-600 px-3 py-2 text-center text-sm font-medium text-white transition-colors hover:bg-emerald-500"
            >
              Comenzar gratis
            </a>
          </div>
        </aside>
      </div>

      {/* Spacer for the fixed header */}
      <div className="h-[72px]" />
    </>
  );
}