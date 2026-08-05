import { SignJWT, jwtVerify } from 'jose';
import bcrypt from 'bcryptjs';
import type { AstroCookies } from 'astro';

export interface CustomerJwtPayload {
  sub: string;
  email: string;
  role: string;
  type: 'customer';
  dbUrl: string;
  dbToken: string;
  academyName: string;
  iat: number;
  exp: number;
}

export interface AdminJwtPayload {
  sub: string;
  email: string;
  role: string;
  type: 'admin';
  iat: number;
  exp: number;
}

export type JwtPayload = CustomerJwtPayload | AdminJwtPayload;

const COOKIE_NAME = 'auth_token';
const TOKEN_EXPIRY = '7d';
const BCRYPT_COST = 12;

function getSecret(): Uint8Array {
  const secret = import.meta.env.JWT_SECRET;
  if (!secret) {
    throw new Error('JWT_SECRET environment variable is not set');
  }
  return new TextEncoder().encode(secret);
}

export async function signToken(
  payload: Omit<CustomerJwtPayload, 'iat' | 'exp'> | Omit<AdminJwtPayload, 'iat' | 'exp'>
): Promise<string> {
  const secret = getSecret();

  return new SignJWT({ ...payload })
    .setProtectedHeader({ alg: 'HS256' })
    .setIssuedAt()
    .setExpirationTime(TOKEN_EXPIRY)
    .sign(secret);
}

export async function verifyToken(token: string): Promise<JwtPayload> {
  const secret = getSecret();
  const { payload } = await jwtVerify(token, secret, {
    algorithms: ['HS256'],
  });

  return payload as unknown as JwtPayload;
}

export async function hashPassword(password: string): Promise<string> {
  return bcrypt.hash(password, BCRYPT_COST);
}

export async function verifyPassword(
  password: string,
  hash: string
): Promise<boolean> {
  return bcrypt.compare(password, hash);
}

export function setAuthCookie(cookies: AstroCookies, token: string): void {
  cookies.set(COOKIE_NAME, token, {
    httpOnly: true,
    secure: true,
    sameSite: 'lax',
    path: '/',
    maxAge: 60 * 60 * 24 * 7, // 7 days
  });
}

export function clearAuthCookie(cookies: AstroCookies): void {
  cookies.delete(COOKIE_NAME, { path: '/' });
}

export function getAuthCookie(cookies: AstroCookies): string | undefined {
  return cookies.get(COOKIE_NAME)?.value;
}
