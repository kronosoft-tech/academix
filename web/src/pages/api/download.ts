import type { APIRoute } from 'astro';
import { db } from '../../lib/db';

const GITHUB_ORG = 'kronosoft-tech';
const GITHUB_REPO = 'academix';

const VALID_OS = ['windows', 'macos', 'linux'] as const;
const VALID_ARCH = ['x64', 'arm64'] as const;

/**
 * Patterns to match the correct asset for each OS+arch combo.
 * These match what tauri-apps/tauri-action generates.
 */
const ASSET_PATTERNS: Record<string, RegExp> = {
  'windows-x64': /x64-setup\.exe$/,
  'windows-arm64': /arm64-setup\.exe$/,
  'macos-x64': /_x64\.dmg$/,
  'macos-arm64': /_aarch64\.dmg$/,
  'linux-x64': /_amd64\.deb$/,
};

interface GitHubAsset {
  name: string;
  browser_download_url: string;
}

interface GitHubRelease {
  tag_name: string;
  assets: GitHubAsset[];
}

let cachedRelease: { data: GitHubRelease; fetchedAt: number } | null = null;
const CACHE_TTL = 5 * 60 * 1000; // 5 minutes

async function getLatestRelease(): Promise<GitHubRelease> {
  if (cachedRelease && Date.now() - cachedRelease.fetchedAt < CACHE_TTL) {
    return cachedRelease.data;
  }

  const res = await fetch(
    `https://api.github.com/repos/${GITHUB_ORG}/${GITHUB_REPO}/releases/latest`,
    { headers: { Accept: 'application/vnd.github+json', 'User-Agent': 'Academix-Web' } }
  );

  if (!res.ok) {
    throw new Error(`GitHub API error: ${res.status}`);
  }

  const data: GitHubRelease = await res.json();
  cachedRelease = { data, fetchedAt: Date.now() };
  return data;
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

  const pattern = ASSET_PATTERNS[`${os}-${arch}`];
  if (!pattern) {
    return new Response(
      JSON.stringify({ error: 'No download available for this platform' }),
      { status: 404, headers: { 'Content-Type': 'application/json' } }
    );
  }

  let release: GitHubRelease;
  try {
    release = await getLatestRelease();
  } catch {
    return new Response(
      JSON.stringify({ error: 'Could not fetch latest release from GitHub' }),
      { status: 502, headers: { 'Content-Type': 'application/json' } }
    );
  }

  const asset = release.assets.find((a) => pattern.test(a.name));
  if (!asset) {
    return new Response(
      JSON.stringify({ error: `No asset found for ${os}-${arch} in release ${release.tag_name}` }),
      { status: 404, headers: { 'Content-Type': 'application/json' } }
    );
  }

  // Track download (fire-and-forget)
  const ip = request.headers.get('x-forwarded-for')?.split(',')[0]?.trim() || 'unknown';
  const country = request.headers.get('x-vercel-ip-country') || 'unknown';

  db.execute({
    sql: 'INSERT INTO downloads (id, os, arch, version, ip, country, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)',
    args: [crypto.randomUUID(), os, arch, release.tag_name, ip, country, new Date().toISOString()],
  }).catch(() => { });

  return new Response(null, {
    status: 302,
    headers: { Location: asset.browser_download_url },
  });
};
