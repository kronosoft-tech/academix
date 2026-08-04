import type { APIRoute } from 'astro';
import { db } from '../../lib/db';

const GITHUB_ORG = 'academix-app';
const GITHUB_REPO = 'academix';
const ALLOW_LIST_PATTERN = /^https:\/\/github\.com\/academix-app\/academix\/releases\/download\/.+/;

const VALID_OS = ['windows', 'macos', 'linux'] as const;
const VALID_ARCH = ['x64', 'arm64'] as const;

const EXTENSIONS: Record<string, string> = {
  windows: 'msi',
  macos: 'dmg',
  linux: 'deb',
};

export const GET: APIRoute = async ({ request, url }) => {
  const os = url.searchParams.get('os');
  const arch = url.searchParams.get('arch');
  const version = url.searchParams.get('version') || 'latest';

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

  const ext = EXTENSIONS[os];
  const tag = version === 'latest' ? 'latest' : `v${version}`;
  const downloadUrl = `https://github.com/${GITHUB_ORG}/${GITHUB_REPO}/releases/download/${tag}/academix-${os}-${arch}.${ext}`;

  if (!ALLOW_LIST_PATTERN.test(downloadUrl)) {
    return new Response(
      JSON.stringify({ error: 'Generated URL does not match allow-list' }),
      { status: 400, headers: { 'Content-Type': 'application/json' } }
    );
  }

  // Track download (fire-and-forget — don't block redirect)
  const ip = request.headers.get('x-forwarded-for')?.split(',')[0]?.trim() || 'unknown';
  const country = request.headers.get('x-vercel-ip-country') || 'unknown';

  db.execute({
    sql: 'INSERT INTO downloads (id, os, arch, version, ip, country, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)',
    args: [crypto.randomUUID(), os, arch, version, ip, country, new Date().toISOString()],
  }).catch(() => { }); // Silently fail — tracking should never block downloads

  return new Response(null, {
    status: 302,
    headers: { Location: downloadUrl },
  });
};
