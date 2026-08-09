import type { UpdateInfo, ArtifactEntry } from '../types/updater';

/**
 * Compares two SemVer strings. Returns true when remote > current.
 * Compares MAJOR, MINOR, PATCH segments numerically left to right.
 */
export function compareSemVer(current: string, remote: string): boolean {
  const currentParts = current.split('.').map(Number);
  const remoteParts = remote.split('.').map(Number);

  for (let i = 0; i < 3; i++) {
    const c = currentParts[i] ?? 0;
    const r = remoteParts[i] ?? 0;

    if (r > c) return true;
    if (r < c) return false;
  }

  return false;
}

/**
 * Validates an unknown payload as an UpdateManifest.
 * Returns UpdateInfo on success or Error on failure.
 *
 * Expected shape: { version, notes, pub_date, platforms: { "target-arch": { url, signature } } }
 */
export function parseUpdateManifest(payload: unknown): UpdateInfo | Error {
  if (payload === null || typeof payload !== 'object') {
    return new Error('Payload must be a non-null object');
  }

  const obj = payload as Record<string, unknown>;

  // Validate version (required, must match SemVer pattern)
  if (typeof obj.version !== 'string') {
    return new Error('Missing or invalid "version" field');
  }

  const semverRegex = /^\d+\.\d+\.\d+$/;
  if (!semverRegex.test(obj.version)) {
    return new Error('Version does not match SemVer format (MAJOR.MINOR.PATCH)');
  }

  // Validate platforms (required, must be an object with at least one entry with signature)
  if (obj.platforms === null || typeof obj.platforms !== 'object') {
    return new Error('Missing or invalid "platforms" field');
  }

  const platforms = obj.platforms as Record<string, unknown>;
  const platformKeys = Object.keys(platforms);

  if (platformKeys.length === 0) {
    return new Error('Platforms object must have at least one entry');
  }

  const hasValidPlatform = platformKeys.some((key) => {
    const entry = platforms[key];
    if (entry === null || typeof entry !== 'object') return false;
    const artifact = entry as Record<string, unknown>;
    return typeof artifact.signature === 'string' && artifact.signature.length > 0;
  });

  if (!hasValidPlatform) {
    return new Error(
      'At least one platform entry must have a non-empty "signature" field'
    );
  }

  // Map fields to UpdateInfo
  const releaseNotes =
    typeof obj.notes === 'string' ? obj.notes : '';
  const date = typeof obj.pub_date === 'string' ? obj.pub_date : '';

  return {
    version: obj.version,
    releaseNotes,
    date,
    mandatory: false,
  };
}

/**
 * Constructs a platform key from target and architecture identifiers.
 * Produces the format: `{target}-{arch}`
 */
export function constructPlatformKey(target: string, arch: string): string {
  return `${target}-${arch}`;
}

/**
 * Looks up a platform key in the manifest platforms object.
 * Returns the ArtifactEntry if found, or null otherwise.
 * Uses Object.hasOwn to avoid prototype pollution (e.g., "__proto__").
 */
export function matchPlatform(
  manifest: Record<string, ArtifactEntry>,
  platformKey: string
): ArtifactEntry | null {
  if (!Object.prototype.hasOwnProperty.call(manifest, platformKey)) return null;
  return manifest[platformKey] ?? null;
}

/**
 * Validates a URL: must be non-empty, <= 2048 characters, and start with https://
 */
export function validateUrl(url: string): boolean {
  if (url.length === 0) return false;
  if (url.length > 2048) return false;
  if (!url.startsWith('https://')) return false;
  return true;
}

/**
 * Truncates release notes to a maximum character length.
 * Default max is 5000 characters.
 */
export function truncateReleaseNotes(notes: string, max: number = 5000): string {
  if (notes.length <= max) return notes;
  return notes.slice(0, max);
}

/**
 * Validates a check interval in hours.
 * Accepts only integers in the range [1, 24].
 */
export function validateCheckInterval(hours: number): boolean {
  if (!Number.isInteger(hours)) return false;
  return hours >= 1 && hours <= 24;
}

/**
 * Validates a public key for Ed25519.
 * Must be non-empty, base64-encoded, and decode to at least 32 bytes.
 * Typically 44+ characters in base64.
 */
export function validatePublicKey(key: string): boolean {
  if (key.length === 0) return false;

  // Check base64 format (standard or URL-safe, with optional padding)
  const base64Regex = /^[A-Za-z0-9+/\-_]+=*$/;
  if (!base64Regex.test(key)) return false;

  // Decode and check minimum length (32 bytes for Ed25519)
  try {
    const decoded = atob(key.replace(/-/g, '+').replace(/_/g, '/'));
    return decoded.length >= 32;
  } catch {
    return false;
  }
}
