import { describe, it, expect } from 'vitest';
import { validatePassword, validatePasswordStrength } from './validatePassword';

describe('validatePassword', () => {
  it('should return valid for valid passwords', () => {
    expect(validatePassword('123456').valid).toBe(true);
    expect(validatePassword('password').valid).toBe(true);
    expect(validatePassword('SecurePass123').valid).toBe(true);
  });

  it('should return error for empty password', () => {
    const result = validatePassword('');
    expect(result.valid).toBe(false);
    expect(result.message).toBe('La contraseña es requerida');
  });

  it('should return error for short password', () => {
    const result = validatePassword('12345');
    expect(result.valid).toBe(false);
    expect(result.message).toBe('La contraseña debe tener al menos 6 caracteres');
  });
});

describe('validatePasswordStrength', () => {
  it('should return valid for strong passwords', () => {
    const result = validatePasswordStrength('SecurePass123!');
    expect(result.valid).toBe(true);
    expect(result.score).toBe(5);
    expect(result.message).toBe('Contraseña segura');
  });

  it('should return error for weak passwords', () => {
    const result = validatePasswordStrength('12345'); // Only numbers, no letters
    expect(result.valid).toBe(false);
    expect(result.score).toBeLessThan(3);
  });

  it('should return medium strength for moderate passwords', () => {
    const result = validatePasswordStrength('password');
    // Only has lowercase and length < 8
    expect(result.score).toBeGreaterThanOrEqual(1);
  });

  it('should handle empty password', () => {
    const result = validatePasswordStrength('');
    expect(result.valid).toBe(false);
    expect(result.score).toBe(0);
    expect(result.message).toBe('La contraseña es requerida');
  });
});
