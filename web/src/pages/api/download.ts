import type { APIRoute } from 'astro';
import { db } from '../../lib/db';

const GITHUB_ORG = 'kronosoft-tech';
const GITHUB_REPO = 'academix';
const VERSION = '0.2.0';

const VALID_OS = ['windows', 'macos', 'linux'] as const;
const VALID_ARCH = ['x64', 'arm64'] as const;

/**
 * Maps OS + arch to the actual filename in GitHub Releases.
 * These match what tauri-apps/tauri-action generates.
 */
function getAssetFilename(os: string, arch: string): string {
  switch (os) {
    case 'windows':
      if (arch === 'arm64') return `academix_${VERSION}_arm64-setup.exe`;
      return `academix_${VERSION}_x64-setup.exe`;
    case 'macos':
      if (arch === 'arm64') return `academix_${VERSION}_aarch64.dmg`;
      return `academix_${VERSION}_x64.dmg`;
    case 'linux':
      return `academix_${VERSION}_amd64.deb`;
    default:
      return '';
  }
}

export const GET: APIRoute = async ({ request, url }) => {
  const os = url.searchParams.get('os');
  const arch = url.searchParams.get('arch');

  if (!os || !arch) {
    return new Response(
      JSON.stringify({ error: 'Missing required parameters: os, arch' }),
      { status: 400, headers: { 'Content-Type': 'application/json' } }
    );
  }

  if (!VALID_OS.includes(os as (typeof VALID_OS)[number])) {
    return new Response(
      JSON.stringify({ error: `Invalid os. Must be one of: ${VALID_OS.join(', ')}` }),
      { status: 400, headers: { 'Content-Type': 'application/json' } }
    );
  }

  if (!VALID_ARCH.includes(arch as (typeof VALID_ARCH)[number])) {
    return new Response(
      JSON.stringify({ error: `Invalid arch. Must be one of: ${VALID_ARCH.join(', ')}` }),
      { status: 400, headers: { 'Content-Type': 'application/json' } }
    );
  }

  const filename = getAssetFilename(os, arch);
  const downloadUrl = `https://github.com/${GITHUB_ORG}/${GITHUB_REPO}/releases/download/app-v${VERSION}/${filename}`;

  // Track download (fire-and-forget)
  const ip = request.headers.get('x-forwarded-for')?.split(',')[0]?.trim() || 'unknown';
  const country = request.headers.get('x-vercel-ip-country') || 'unknown';

  db.execute({
    sql: 'INSERT INTO downloads (id, os, arch, version, ip, country, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)',
    args: [crypto.randomUUID(), os, arch, VERSION, ip, country, new Date().toISOString()],
  }).catch(() => { }); // Silently fail

  return new Response(null, {
    status: 302,
    headers: { Location: downloadUrl },
  });
};
