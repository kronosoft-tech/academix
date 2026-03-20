import { describe, it, expect } from 'vitest';
import { validateEmail, validateEmailWithMessage } from './validateEmail';

describe('validateEmail', () => {
  it('should return true for valid emails', () => {
    expect(validateEmail('test@example.com')).toBe(true);
    expect(validateEmail('user.name@domain.org')).toBe(true);
    expect(validateEmail('user+tag@example.co.uk')).toBe(true);
  });

  it('should return false for invalid emails', () => {
    expect(validateEmail('')).toBe(false);
    expect(validateEmail('notanemail')).toBe(false);
    expect(validateEmail('@nodomain.com')).toBe(false);
    expect(validateEmail('no@domain')).toBe(false);
    expect(validateEmail('spaces in@email.com')).toBe(false);
  });
});

describe('validateEmailWithMessage', () => {
  it('should return valid for correct email', () => {
    const result = validateEmailWithMessage('test@example.com');
    expect(result.valid).toBe(true);
    expect(result.message).toBe('');
  });

  it('should return error for empty email', () => {
    const result = validateEmailWithMessage('');
    expect(result.valid).toBe(false);
    expect(result.message).toBe('El correo electrónico es requerido');
  });

  it('should return error for invalid email format', () => {
    const result = validateEmailWithMessage('invalid');
    expect(result.valid).toBe(false);
    expect(result.message).toBe('Ingrese un correo electrónico válido');
  });
});
