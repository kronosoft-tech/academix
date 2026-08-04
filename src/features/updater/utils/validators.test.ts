import { describe, it, expect } from 'vitest';
import fc from 'fast-check';
import { compareSemVer, constructPlatformKey, parseUpdateManifest, matchPlatform, validateUrl, validateCheckInterval, truncateReleaseNotes, validatePublicKey } from './validators';
import type { ArtifactEntry } from '../types/updater';

/**
 * Property 1: SemVer Comparison Correctness
 *
 * For any two valid SemVer version strings A and B, where B is semantically
 * higher than A, the `compareSemVer` function SHALL return `true`, and for
 * any pair where B is equal to or lower than A, it SHALL return `false`.
 *
 * **Validates: Requirements 1.3**
 */
describe('Feature: auto-updater, Property 1: SemVer Comparison Correctness', () => {
  const versionSegment = fc.integer({ min: 0, max: 999 });

  it('returns true when remote major > current major', () => {
    fc.assert(
      fc.property(
        versionSegment,
        versionSegment,
        versionSegment,
        fc.integer({ min: 1, max: 999 }),
        versionSegment,
        versionSegment,
        (currentMajor, currentMinor, currentPatch, majorDelta, remoteMinor, remotePatch) => {
          fc.pre(currentMajor + majorDelta <= 999);
          const remoteMajor = currentMajor + majorDelta;
          const current = `${currentMajor}.${currentMinor}.${currentPatch}`;
          const remote = `${remoteMajor}.${remoteMinor}.${remotePatch}`;
          expect(compareSemVer(current, remote)).toBe(true);
        }
      ),
      { numRuns: 100 }
    );
  });

  it('returns true when same major but remote minor > current minor', () => {
    fc.assert(
      fc.property(
        versionSegment,
        versionSegment,
        versionSegment,
        fc.integer({ min: 1, max: 999 }),
        versionSegment,
        (major, currentMinor, currentPatch, minorDelta, remotePatch) => {
          fc.pre(currentMinor + minorDelta <= 999);
          const remoteMinor = currentMinor + minorDelta;
          const current = `${major}.${currentMinor}.${currentPatch}`;
          const remote = `${major}.${remoteMinor}.${remotePatch}`;
          expect(compareSemVer(current, remote)).toBe(true);
        }
      ),
      { numRuns: 100 }
    );
  });

  it('returns true when same major and minor but remote patch > current patch', () => {
    fc.assert(
      fc.property(
        versionSegment,
        versionSegment,
        versionSegment,
        fc.integer({ min: 1, max: 999 }),
        (major, minor, currentPatch, patchDelta) => {
          fc.pre(currentPatch + patchDelta <= 999);
          const remotePatch = currentPatch + patchDelta;
          const current = `${major}.${minor}.${currentPatch}`;
          const remote = `${major}.${minor}.${remotePatch}`;
          expect(compareSemVer(current, remote)).toBe(true);
        }
      ),
      { numRuns: 100 }
    );
  });

  it('returns false when remote == current (reflexivity)', () => {
    fc.assert(
      fc.property(
        versionSegment,
        versionSegment,
        versionSegment,
        (major, minor, patch) => {
          const version = `${major}.${minor}.${patch}`;
          expect(compareSemVer(version, version)).toBe(false);
        }
      ),
      { numRuns: 100 }
    );
  });

  it('returns false when remote < current', () => {
    fc.assert(
      fc.property(
        versionSegment,
        versionSegment,
        versionSegment,
        versionSegment,
        versionSegment,
        versionSegment,
        (currentMajor, currentMinor, currentPatch, remoteMajor, remoteMinor, remotePatch) => {
          const current = `${currentMajor}.${currentMinor}.${currentPatch}`;
          const remote = `${remoteMajor}.${remoteMinor}.${remotePatch}`;
          // Pre-condition: remote must be strictly less than current
          const cParts = [currentMajor, currentMinor, currentPatch];
          const rParts = [remoteMajor, remoteMinor, remotePatch];
          let isRemoteLess = false;
          for (let i = 0; i < 3; i++) {
            if (rParts[i]! < cParts[i]!) { isRemoteLess = true; break; }
            if (rParts[i]! > cParts[i]!) break;
          }
          fc.pre(isRemoteLess);
          expect(compareSemVer(current, remote)).toBe(false);
        }
      ),
      { numRuns: 100 }
    );
  });
});

/**
 * Property 3: Platform Key Construction
 *
 * For any valid target identifier (windows, darwin, linux) and any valid
 * architecture identifier (x86_64, aarch64, i686, armv7), the platform key
 * constructor SHALL produce a string in the exact format `{target}-{arch}`.
 *
 * **Validates: Requirements 2.1, 2.3, 7.7**
 */
describe('Feature: auto-updater, Property 3: Platform Key Construction', () => {
  const targets = fc.constantFrom('windows', 'darwin', 'linux');
  const archs = fc.constantFrom('x86_64', 'aarch64', 'i686', 'armv7');

  it('produces exactly `${target}-${arch}` for any valid target and arch', () => {
    fc.assert(
      fc.property(targets, archs, (target, arch) => {
        const result = constructPlatformKey(target, arch);
        expect(result).toBe(`${target}-${arch}`);
      }),
      { numRuns: 100 }
    );
  });

  it('result always contains exactly one hyphen', () => {
    fc.assert(
      fc.property(targets, archs, (target, arch) => {
        const result = constructPlatformKey(target, arch);
        const hyphenCount = [...result].filter((c) => c === '-').length;
        expect(hyphenCount).toBe(1);
      }),
      { numRuns: 100 }
    );
  });

  it('target appears before the hyphen, arch appears after', () => {
    fc.assert(
      fc.property(targets, archs, (target, arch) => {
        const result = constructPlatformKey(target, arch);
        const hyphenIndex = result.indexOf('-');
        const beforeHyphen = result.slice(0, hyphenIndex);
        const afterHyphen = result.slice(hyphenIndex + 1);
        expect(beforeHyphen).toBe(target);
        expect(afterHyphen).toBe(arch);
      }),
      { numRuns: 100 }
    );
  });
});


/**
 * Property 4: Platform Matching Exclusion
 *
 * For any Update_Manifest whose `platforms` object does not contain the current
 * Platform_Target as a key, the platform matcher SHALL return null, regardless
 * of the number of other platform entries present.
 *
 * **Validates: Requirements 2.2**
 */
describe('Feature: auto-updater, Property 4: Platform Matching Exclusion', () => {
  const artifactEntry: fc.Arbitrary<ArtifactEntry> = fc.record({
    url: fc.webUrl(),
    signature: fc.string({ minLength: 1, maxLength: 100 }),
  });

  const platformKey = fc.string({ minLength: 3, maxLength: 20 });

  it('when the manifest does NOT contain the given platform key, returns null', () => {
    fc.assert(
      fc.property(
        platformKey,
        fc.array(platformKey, { minLength: 0, maxLength: 5 }),
        fc.array(artifactEntry, { minLength: 0, maxLength: 5 }),
        (targetKey, otherKeys, artifacts) => {
          // Build a manifest from otherKeys, ensuring target is NOT present
          const filteredKeys = otherKeys.filter((k) => k !== targetKey);
          const manifest: Record<string, ArtifactEntry> = {};
          for (let i = 0; i < filteredKeys.length; i++) {
            if (artifacts[i]) {
              manifest[filteredKeys[i]] = artifacts[i];
            }
          }

          const result = matchPlatform(manifest, targetKey);
          expect(result).toBeNull();
        }
      ),
      { numRuns: 100 }
    );
  });

  it('when the manifest DOES contain the given platform key, returns the matching ArtifactEntry', () => {
    fc.assert(
      fc.property(
        platformKey,
        artifactEntry,
        fc.array(
          fc.tuple(platformKey, artifactEntry),
          { minLength: 0, maxLength: 5 }
        ),
        (targetKey, targetArtifact, extras) => {
          const manifest: Record<string, ArtifactEntry> = {};

          // Add extra entries (some might overlap with targetKey, but we overwrite below)
          for (const [key, artifact] of extras) {
            manifest[key] = artifact;
          }

          // Ensure the target key is present with the expected artifact
          manifest[targetKey] = targetArtifact;

          const result = matchPlatform(manifest, targetKey);
          expect(result).not.toBeNull();
          expect(result).toEqual(targetArtifact);
        }
      ),
      { numRuns: 100 }
    );
  });

  it('empty manifest always returns null for any platform key', () => {
    fc.assert(
      fc.property(platformKey, (key) => {
        const manifest: Record<string, ArtifactEntry> = {};
        const result = matchPlatform(manifest, key);
        expect(result).toBeNull();
      }),
      { numRuns: 100 }
    );
  });
});


/**
 * Property 5: URL Validation Constraint
 *
 * For any artifact URL string, the URL validator SHALL accept it only when
 * it is non-empty AND its length is 2048 characters or fewer.
 * Empty strings or strings exceeding 2048 characters SHALL always be rejected.
 *
 * **Validates: Requirements 2.5**
 */
describe('Feature: auto-updater, Property 5: URL Validation Constraint', () => {
  // Arbitraries
  const validUrl = fc
    .string({ minLength: 1, maxLength: 2040 })
    .map((s) => `https://${s.replace(/[^a-zA-Z0-9]/g, 'a')}.com/path`);

  const longUrl = fc.string({ minLength: 2049, maxLength: 3000 });

  const httpUrl = fc
    .string({ minLength: 1, maxLength: 100 })
    .map((s) => `http://${s.replace(/[^a-zA-Z0-9]/g, 'a')}.com`);

  it('should always reject empty strings', () => {
    fc.assert(
      fc.property(fc.constant(''), (url) => {
        expect(validateUrl(url)).toBe(false);
      }),
      { numRuns: 100 }
    );
  });

  it('should always reject strings longer than 2048 characters', () => {
    fc.assert(
      fc.property(longUrl, (url) => {
        expect(validateUrl(url)).toBe(false);
      }),
      { numRuns: 100 }
    );
  });

  it('should reject HTTP URLs (not HTTPS)', () => {
    fc.assert(
      fc.property(httpUrl, (url) => {
        expect(validateUrl(url)).toBe(false);
      }),
      { numRuns: 100 }
    );
  });

  it('should accept valid HTTPS URLs with length <= 2048', () => {
    fc.assert(
      fc.property(validUrl, (url) => {
        if (url.length <= 2048) {
          expect(validateUrl(url)).toBe(true);
        }
      }),
      { numRuns: 100 }
    );
  });

  it('should reject strings not starting with https://', () => {
    const nonHttpsString = fc
      .string({ minLength: 1, maxLength: 200 })
      .filter((s) => !s.startsWith('https://'));

    fc.assert(
      fc.property(nonHttpsString, (url) => {
        expect(validateUrl(url)).toBe(false);
      }),
      { numRuns: 100 }
    );
  });
});

/**
 * Property 2: Invalid Payload Rejection
 *
 * For any response body that is not valid JSON or lacks the required `version`,
 * `platforms`, and `signature` fields, the manifest parser SHALL return an error
 * result and never produce a valid `UpdateInfo` struct.
 *
 * **Validates: Requirements 1.5, 7.6**
 */
describe('Feature: auto-updater, Property 2: Invalid Payload Rejection', () => {
  const NUM_RUNS = 100;

  it('any null/undefined/primitive payload returns Error', () => {
    const primitives = fc.oneof(
      fc.constant(null),
      fc.constant(undefined),
      fc.boolean(),
      fc.integer(),
      fc.double(),
      fc.string()
    );

    fc.assert(
      fc.property(primitives, (payload) => {
        const result = parseUpdateManifest(payload);
        expect(result).toBeInstanceOf(Error);
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('object missing "version" field returns Error', () => {
    const objectWithoutVersion = fc
      .record({
        notes: fc.string(),
        pub_date: fc.string(),
        platforms: fc
          .dictionary(
            fc.string({ minLength: 1 }),
            fc.record({
              url: fc.string(),
              signature: fc.string({ minLength: 1 }),
            })
          )
          .filter((p) => Object.keys(p).length > 0),
      });

    fc.assert(
      fc.property(objectWithoutVersion, (payload) => {
        const result = parseUpdateManifest(payload);
        expect(result).toBeInstanceOf(Error);
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('object with non-SemVer version string returns Error', () => {
    const semverRegex = /^\d+\.\d+\.\d+$/;
    const invalidVersions = fc.string().filter((s) => !semverRegex.test(s));

    const objectWithBadVersion = fc.record({
      version: invalidVersions,
      notes: fc.string(),
      pub_date: fc.string(),
      platforms: fc
        .dictionary(
          fc.string({ minLength: 1 }),
          fc.record({
            url: fc.string(),
            signature: fc.string({ minLength: 1 }),
          })
        )
        .filter((p) => Object.keys(p).length > 0),
    });

    fc.assert(
      fc.property(objectWithBadVersion, (payload) => {
        const result = parseUpdateManifest(payload);
        expect(result).toBeInstanceOf(Error);
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('object missing "platforms" field returns Error', () => {
    const validVersion = fc
      .tuple(
        fc.integer({ min: 0, max: 99 }),
        fc.integer({ min: 0, max: 99 }),
        fc.integer({ min: 0, max: 99 })
      )
      .map(([a, b, c]) => `${a}.${b}.${c}`);

    const objectWithoutPlatforms = fc.record({
      version: validVersion,
      notes: fc.string(),
      pub_date: fc.string(),
    });

    fc.assert(
      fc.property(objectWithoutPlatforms, (payload) => {
        const result = parseUpdateManifest(payload);
        expect(result).toBeInstanceOf(Error);
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('object with empty platforms returns Error', () => {
    const validVersion = fc
      .tuple(
        fc.integer({ min: 0, max: 99 }),
        fc.integer({ min: 0, max: 99 }),
        fc.integer({ min: 0, max: 99 })
      )
      .map(([a, b, c]) => `${a}.${b}.${c}`);

    const objectWithEmptyPlatforms = fc.record({
      version: validVersion,
      notes: fc.string(),
      pub_date: fc.string(),
      platforms: fc.constant({}),
    });

    fc.assert(
      fc.property(objectWithEmptyPlatforms, (payload) => {
        const result = parseUpdateManifest(payload);
        expect(result).toBeInstanceOf(Error);
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('object with platforms but no entry has a signature returns Error', () => {
    const validVersion = fc
      .tuple(
        fc.integer({ min: 0, max: 99 }),
        fc.integer({ min: 0, max: 99 }),
        fc.integer({ min: 0, max: 99 })
      )
      .map(([a, b, c]) => `${a}.${b}.${c}`);

    const platformWithNoSignature = fc
      .dictionary(
        fc.string({ minLength: 1 }),
        fc.record({
          url: fc.string(),
          signature: fc.constant(''),
        })
      )
      .filter((p) => Object.keys(p).length > 0);

    const objectWithNoSignatures = fc.record({
      version: validVersion,
      notes: fc.string(),
      pub_date: fc.string(),
      platforms: platformWithNoSignature,
    });

    fc.assert(
      fc.property(objectWithNoSignatures, (payload) => {
        const result = parseUpdateManifest(payload);
        expect(result).toBeInstanceOf(Error);
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('valid manifest returns UpdateInfo (not Error)', () => {
    const validManifest = fc.record({
      version: fc
        .tuple(
          fc.integer({ min: 0, max: 99 }),
          fc.integer({ min: 0, max: 99 }),
          fc.integer({ min: 0, max: 99 })
        )
        .map(([a, b, c]) => `${a}.${b}.${c}`),
      notes: fc.string(),
      pub_date: fc.string(),
      platforms: fc
        .dictionary(
          fc.string({ minLength: 1 }),
          fc.record({
            url: fc.string(),
            signature: fc.string({ minLength: 1 }),
          })
        )
        .filter((p) => Object.keys(p).length > 0),
    });

    fc.assert(
      fc.property(validManifest, (payload) => {
        const result = parseUpdateManifest(payload);
        expect(result).not.toBeInstanceOf(Error);
        if (!(result instanceof Error)) {
          expect(result.version).toBe(payload.version);
          expect(result.releaseNotes).toBe(payload.notes);
          expect(result.date).toBe(payload.pub_date);
        }
      }),
      { numRuns: NUM_RUNS }
    );
  });
});


/**
 * Property 8: Check Interval Persistence Round-Trip
 *
 * For any valid interval value between 1 and 24 (inclusive), the validation
 * function SHALL accept it. Values outside the range [1, 24] SHALL be rejected.
 *
 * **Validates: Requirements 1.2**
 */
describe('Feature: auto-updater, Property 8: Check Interval Persistence Round-Trip', () => {
  const validInterval = fc.integer({ min: 1, max: 24 });
  const tooSmall = fc.integer({ min: -1000, max: 0 });
  const tooLarge = fc.integer({ min: 25, max: 1000 });
  const floats = fc.double({ min: 1, max: 24 }).filter((n) => !Number.isInteger(n));

  it('any integer in [1, 24] returns true', () => {
    fc.assert(
      fc.property(validInterval, (hours) => {
        expect(validateCheckInterval(hours)).toBe(true);
      }),
      { numRuns: 100 }
    );
  });

  it('any integer < 1 returns false', () => {
    fc.assert(
      fc.property(tooSmall, (hours) => {
        expect(validateCheckInterval(hours)).toBe(false);
      }),
      { numRuns: 100 }
    );
  });

  it('any integer > 24 returns false', () => {
    fc.assert(
      fc.property(tooLarge, (hours) => {
        expect(validateCheckInterval(hours)).toBe(false);
      }),
      { numRuns: 100 }
    );
  });

  it('non-integer numbers (floats) return false', () => {
    fc.assert(
      fc.property(floats, (hours) => {
        expect(validateCheckInterval(hours)).toBe(false);
      }),
      { numRuns: 100 }
    );
  });

  it('0 returns false', () => {
    expect(validateCheckInterval(0)).toBe(false);
  });
});


/**
 * Property 6: Release Notes Truncation
 *
 * For any release notes string, the notification formatter SHALL produce output
 * containing at most 5000 characters. Notes longer than 5000 characters SHALL
 * be truncated to exactly 5000 characters.
 *
 * **Validates: Requirements 3.1**
 */
describe('Feature: auto-updater, Property 6: Release Notes Truncation', () => {

  const shortNotes = fc.string({ minLength: 0, maxLength: 5000 });
  const longNotes = fc.string({ minLength: 5001, maxLength: 10000 });
  const customMax = fc.integer({ min: 1, max: 10000 });

  it('for any string of length <= 5000, output equals input (no truncation)', () => {
    fc.assert(
      fc.property(shortNotes, (notes) => {
        const result = truncateReleaseNotes(notes);
        expect(result).toBe(notes);
      }),
      { numRuns: 100 }
    );
  });

  it('for any string of length > 5000, output has exactly 5000 characters', () => {
    fc.assert(
      fc.property(longNotes, (notes) => {
        const result = truncateReleaseNotes(notes);
        expect(result.length).toBe(5000);
      }),
      { numRuns: 100 }
    );
  });

  it('truncated output is always a prefix of the original string', () => {
    fc.assert(
      fc.property(longNotes, (notes) => {
        const result = truncateReleaseNotes(notes);
        expect(notes.startsWith(result)).toBe(true);
      }),
      { numRuns: 100 }
    );
  });

  it('output length is always <= max (default 5000)', () => {
    const anyNotes = fc.string({ minLength: 0, maxLength: 10000 });

    fc.assert(
      fc.property(anyNotes, (notes) => {
        const result = truncateReleaseNotes(notes);
        expect(result.length).toBeLessThanOrEqual(5000);
      }),
      { numRuns: 100 }
    );
  });

  it('custom max parameter is respected', () => {
    const anyNotes = fc.string({ minLength: 0, maxLength: 10000 });

    fc.assert(
      fc.property(anyNotes, customMax, (notes, max) => {
        const result = truncateReleaseNotes(notes, max);
        expect(result.length).toBeLessThanOrEqual(max);
        if (notes.length <= max) {
          expect(result).toBe(notes);
        } else {
          expect(result.length).toBe(max);
          expect(notes.startsWith(result)).toBe(true);
        }
      }),
      { numRuns: 100 }
    );
  });
});


/**
 * Property 9: HTTPS-Only Endpoint Enforcement
 *
 * For any endpoint URL string, the endpoint validator SHALL accept only URLs
 * whose scheme is `https`. Any URL with scheme `http` or any other non-TLS
 * scheme SHALL be rejected.
 *
 * **Validates: Requirements 7.2**
 */
describe('Feature: auto-updater, Property 9: HTTPS-Only Endpoint Enforcement', () => {
  const NUM_RUNS = 100;

  // Arbitraries
  const httpsUrl = fc
    .string({ minLength: 1, maxLength: 2030 })
    .map((s) => `https://${s.replace(/[^a-zA-Z0-9.-]/g, 'a')}`);

  const httpUrl = fc
    .string({ minLength: 1, maxLength: 100 })
    .map((s) => `http://${s.replace(/[^a-zA-Z0-9.-]/g, 'a')}`);

  const ftpUrl = fc
    .string({ minLength: 1, maxLength: 100 })
    .map((s) => `ftp://${s.replace(/[^a-zA-Z0-9.-]/g, 'a')}`);

  const noScheme = fc
    .string({ minLength: 1, maxLength: 100 })
    .filter((s) => !s.startsWith('https://'));

  it('URLs starting with https:// and length <= 2048 are accepted', () => {
    fc.assert(
      fc.property(httpsUrl, (url) => {
        if (url.length <= 2048) {
          expect(validateUrl(url)).toBe(true);
        }
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('URLs starting with http:// are always rejected', () => {
    fc.assert(
      fc.property(httpUrl, (url) => {
        expect(validateUrl(url)).toBe(false);
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('URLs starting with ftp:// are always rejected', () => {
    fc.assert(
      fc.property(ftpUrl, (url) => {
        expect(validateUrl(url)).toBe(false);
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('URLs with no recognized scheme are always rejected', () => {
    fc.assert(
      fc.property(noScheme, (url) => {
        expect(validateUrl(url)).toBe(false);
      }),
      { numRuns: NUM_RUNS }
    );
  });
});


/**
 * Property 11: Public Key Validation
 *
 * For any string that is not a valid Ed25519 public key (empty, wrong length,
 * invalid encoding), the configuration validator SHALL reject it and prevent
 * update checks from proceeding.
 *
 * **Validates: Requirements 6.1**
 */
describe('Feature: auto-updater, Property 11: Public Key Validation', () => {
  const NUM_RUNS = 100;

  // Valid Ed25519 key (32+ bytes in base64 = 44+ chars)
  const validBase64Key = fc
    .uint8Array({ minLength: 32, maxLength: 64 })
    .map((arr) => btoa(String.fromCharCode(...arr)));

  // Too short (< 32 bytes decoded)
  const shortBase64Key = fc
    .uint8Array({ minLength: 1, maxLength: 31 })
    .map((arr) => btoa(String.fromCharCode(...arr)));

  // Invalid base64 (contains spaces or invalid chars)
  const invalidBase64 = fc
    .string({ minLength: 1, maxLength: 100 })
    .filter((s) => /[^A-Za-z0-9+/\-_=]/.test(s));

  it('empty string is always rejected', () => {
    fc.assert(
      fc.property(fc.constant(''), (key) => {
        expect(validatePublicKey(key)).toBe(false);
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('non-base64 strings (containing invalid chars) are rejected', () => {
    fc.assert(
      fc.property(invalidBase64, (key) => {
        expect(validatePublicKey(key)).toBe(false);
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('valid base64 strings that decode to < 32 bytes are rejected (too short for Ed25519)', () => {
    fc.assert(
      fc.property(shortBase64Key, (key) => {
        expect(validatePublicKey(key)).toBe(false);
      }),
      { numRuns: NUM_RUNS }
    );
  });

  it('valid base64 strings that decode to >= 32 bytes are accepted', () => {
    fc.assert(
      fc.property(validBase64Key, (key) => {
        expect(validatePublicKey(key)).toBe(true);
      }),
      { numRuns: NUM_RUNS }
    );
  });
});
