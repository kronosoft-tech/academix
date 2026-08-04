import { describe, it, expect, vi, beforeAll } from 'vitest';

// Mock import.meta.env for tests
vi.stubEnv('JWT_SECRET', 'test-secret-key-at-least-32-chars-long!');

// We need to test the auth module functions directly
// Since they use import.meta.env, we mock at module level

describe('Auth Library', () => {
  let signToken: typeof import('../lib/auth').signToken;
  let verifyToken: typeof import('../lib/auth').verifyToken;
  let hashPassword: typeof import('../lib/auth').hashPassword;
  let verifyPassword: typeof import('../lib/auth').verifyPassword;

  beforeAll(async () => {
    const auth = await import('../lib/auth');
    signToken = auth.signToken;
    verifyToken = auth.verifyToken;
    hashPassword = auth.hashPassword;
    verifyPassword = auth.verifyPassword;
  });

  describe('JWT sign/verify', () => {
    it('should sign and verify a customer token', async () => {
      const payload = {
        sub: 'user-123',
        email: 'customer@test.com',
        role: 'Admin',
        type: 'customer' as const,
      };

      const token = await signToken(payload);
      expect(token).toBeDefined();
      expect(typeof token).toBe('string');

      const verified = await verifyToken(token);
      expect(verified.sub).toBe('user-123');
      expect(verified.email).toBe('customer@test.com');
      expect(verified.role).toBe('Admin');
      expect(verified.type).toBe('customer');
      expect(verified.iat).toBeDefined();
      expect(verified.exp).toBeDefined();
    });

    it('should sign and verify an admin token', async () => {
      const payload = {
        sub: 'admin-456',
        email: 'admin@academix.dev',
        role: 'superadmin',
        type: 'admin' as const,
      };

      const token = await signToken(payload);
      expect(token).toBeDefined();

      const verified = await verifyToken(token);
      expect(verified.sub).toBe('admin-456');
      expect(verified.email).toBe('admin@academix.dev');
      expect(verified.role).toBe('superadmin');
      expect(verified.type).toBe('admin');
    });

    it('should discriminate between customer and admin types', async () => {
      const customerToken = await signToken({
        sub: 'c-1',
        email: 'c@test.com',
        role: 'Gerente',
        type: 'customer' as const,
      });

      const adminToken = await signToken({
        sub: 'a-1',
        email: 'a@test.com',
        role: 'employee',
        type: 'admin' as const,
      });

      const customerPayload = await verifyToken(customerToken);
      const adminPayload = await verifyToken(adminToken);

      expect(customerPayload.type).toBe('customer');
      expect(adminPayload.type).toBe('admin');
      expect(customerPayload.type).not.toBe(adminPayload.type);
    });

    it('should throw on invalid token', async () => {
      await expect(verifyToken('invalid.token.here')).rejects.toThrow();
    });

    it('should throw on tampered token', async () => {
      const token = await signToken({
        sub: 'user-1',
        email: 'test@test.com',
        role: 'Admin',
        type: 'customer' as const,
      });

      // Tamper with the token payload
      const parts = token.split('.');
      parts[1] = 'dGFtcGVyZWQ'; // "tampered" in base64
      const tampered = parts.join('.');

      await expect(verifyToken(tampered)).rejects.toThrow();
    });

    it('should include expiration in the token', async () => {
      const token = await signToken({
        sub: 'user-1',
        email: 'test@test.com',
        role: 'Admin',
        type: 'customer' as const,
      });

      const payload = await verifyToken(token);
      // exp should be roughly 7 days from now
      const expectedExp = Math.floor(Date.now() / 1000) + 7 * 24 * 60 * 60;
      expect(payload.exp).toBeGreaterThan(Math.floor(Date.now() / 1000));
      expect(payload.exp).toBeLessThanOrEqual(expectedExp + 5); // 5s tolerance
    });
  });

  describe('bcrypt hash/verify', () => {
    it('should hash a password and verify it', async () => {
      const password = 'MySecurePassword123!';
      const hash = await hashPassword(password);

      expect(hash).toBeDefined();
      expect(hash).not.toBe(password);
      expect(hash.startsWith('$2')).toBe(true);

      const isValid = await verifyPassword(password, hash);
      expect(isValid).toBe(true);
    });

    it('should reject incorrect password', async () => {
      const hash = await hashPassword('correct-password');
      const isValid = await verifyPassword('wrong-password', hash);
      expect(isValid).toBe(false);
    });

    it('should verify passwords hashed with different bcrypt costs (cross-cost interop)', async () => {
      // Simulate a hash from desktop app (cost 10)
      const bcrypt = await import('bcryptjs');
      const cost10Hash = await bcrypt.hash('shared-password', 10);

      // Web verifies with its library (cost embedded in hash)
      const isValid = await verifyPassword('shared-password', cost10Hash);
      expect(isValid).toBe(true);
    });

    it('should produce cost-12 hashes', async () => {
      const hash = await hashPassword('test-password');
      // bcryptjs encodes cost as $2a$12$ or $2b$12$
      expect(hash).toMatch(/\$2[ab]\$12\$/);
    });
  });
});
