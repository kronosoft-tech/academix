import { describe, it, expect } from 'vitest';

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

function buildDownloadUrl(os: string, arch: string, version: string): string {
  const ext = EXTENSIONS[os];
  const tag = version === 'latest' ? 'latest' : `v${version}`;
  return `https://github.com/${GITHUB_ORG}/${GITHUB_REPO}/releases/download/${tag}/academix-${os}-${arch}.${ext}`;
}

function validateParams(os: string | null, arch: string | null): { valid: boolean; error?: string } {
  if (!os || !arch) {
    return { valid: false, error: 'Missing required parameters: os, arch' };
  }
  if (!VALID_OS.includes(os as (typeof VALID_OS)[number])) {
    return { valid: false, error: `Invalid os. Must be one of: ${VALID_OS.join(', ')}` };
  }
  if (!VALID_ARCH.includes(arch as (typeof VALID_ARCH)[number])) {
    return { valid: false, error: `Invalid arch. Must be one of: ${VALID_ARCH.join(', ')}` };
  }
  return { valid: true };
}

describe('Download endpoint', () => {
  describe('URL construction', () => {
    it('should construct correct URL for Windows x64 latest', () => {
      const url = buildDownloadUrl('windows', 'x64', 'latest');
      expect(url).toBe(
        'https://github.com/academix-app/academix/releases/download/latest/academix-windows-x64.msi'
      );
    });

    it('should construct correct URL for macOS arm64 with version', () => {
      const url = buildDownloadUrl('macos', 'arm64', '1.2.0');
      expect(url).toBe(
        'https://github.com/academix-app/academix/releases/download/v1.2.0/academix-macos-arm64.dmg'
      );
    });

    it('should construct correct URL for Linux x64', () => {
      const url = buildDownloadUrl('linux', 'x64', 'latest');
      expect(url).toBe(
        'https://github.com/academix-app/academix/releases/download/latest/academix-linux-x64.deb'
      );
    });
  });

  describe('Allow-list validation', () => {
    it('should pass allow-list for valid GitHub URL', () => {
      const url = buildDownloadUrl('windows', 'x64', 'latest');
      expect(ALLOW_LIST_PATTERN.test(url)).toBe(true);
    });

    it('should reject URLs not matching the allow-list pattern', () => {
      const maliciousUrl = 'https://evil.com/malware.exe';
      expect(ALLOW_LIST_PATTERN.test(maliciousUrl)).toBe(false);
    });

    it('should reject URLs with wrong GitHub org', () => {
      const wrongOrg = 'https://github.com/hacker/academix/releases/download/latest/academix-windows-x64.msi';
      expect(ALLOW_LIST_PATTERN.test(wrongOrg)).toBe(false);
    });

    it('should reject URLs with wrong repo', () => {
      const wrongRepo = 'https://github.com/academix-app/malware/releases/download/latest/bad.exe';
      expect(ALLOW_LIST_PATTERN.test(wrongRepo)).toBe(false);
    });
  });

  describe('Parameter validation', () => {
    it('should reject missing os', () => {
      const result = validateParams(null, 'x64');
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Missing');
    });

    it('should reject missing arch', () => {
      const result = validateParams('windows', null);
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Missing');
    });

    it('should reject invalid os', () => {
      const result = validateParams('android', 'x64');
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Invalid os');
    });

    it('should reject invalid arch', () => {
      const result = validateParams('windows', 'x86');
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Invalid arch');
    });

    it('should accept valid params', () => {
      const result = validateParams('windows', 'x64');
      expect(result.valid).toBe(true);
    });
  });
});
