import { defineMiddleware } from 'astro:middleware';
import { verifyToken, getAuthCookie } from './lib/auth';

const PUBLIC_ROUTES = [
  '/',
  '/pricing',
  '/downloads',
  '/faq',
  '/contact',
  '/tutorials',
  '/auth/login',
  '/auth/register',
  '/auth/reset-password',
  '/admin/login',
];

function isPublicRoute(pathname: string): boolean {
  if (PUBLIC_ROUTES.includes(pathname)) return true;
  if (pathname.startsWith('/tutorials/')) return true;
  if (pathname.startsWith('/api/')) return true;
  return false;
}

function isDashboardRoute(pathname: string): boolean {
  return pathname.startsWith('/dashboard');
}

function isAdminRoute(pathname: string): boolean {
  return pathname.startsWith('/admin') && pathname !== '/admin/login';
}

export const onRequest = defineMiddleware(async (context, next) => {
  const { pathname } = context.url;

  // Skip middleware for prerendered routes
  if (context.isPrerendered) return next();

  // Public routes pass through
  if (isPublicRoute(pathname)) return next();

  const token = getAuthCookie(context.cookies);

  // No token on protected route — redirect to appropriate login
  if (!token) {
    if (isAdminRoute(pathname)) {
      return context.redirect('/admin/login');
    }
    return context.redirect('/auth/login');
  }

  try {
    const payload = await verifyToken(token);

    // Cross-type access protection
    if (payload.type === 'customer' && isAdminRoute(pathname)) {
      return new Response('Forbidden', { status: 403 });
    }

    if (payload.type === 'admin' && isDashboardRoute(pathname)) {
      return new Response('Forbidden', { status: 403 });
    }

    // Populate locals based on type
    if (payload.type === 'customer') {
      context.locals.user = {
        id: payload.sub,
        email: payload.email,
        role: payload.role,
        type: 'customer',
      };
    } else if (payload.type === 'admin') {
      context.locals.admin = {
        id: payload.sub,
        email: payload.email,
        role: payload.role,
        type: 'admin',
      };
    }

    return next();
  } catch {
    // Invalid or expired token — redirect to login
    if (isAdminRoute(pathname)) {
      return context.redirect('/admin/login');
    }
    return context.redirect('/auth/login');
  }
});
