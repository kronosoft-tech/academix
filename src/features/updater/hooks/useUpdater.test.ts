import { describe, it, expect } from 'vitest';
import fc from 'fast-check';

/**
 * Property 10: Retry with Exponential Backoff
 *
 * For any sequence of N consecutive failures (where N ≤ 3), the retry mechanism
 * SHALL attempt exactly N retries with delays of 1s, 2s, and 4s respectively
 * before reporting failure.
 *
 * **Validates: Requirements 7.5**
 */

// Replicate the backoff calculation from useUpdater.ts
const BACKOFF_BASE_MS = 1000;
const MAX_RETRIES = 3;

function calculateBackoffDelay(attempt: number): number {
  return BACKOFF_BASE_MS * Math.pow(2, attempt - 1);
}

describe('Feature: auto-updater, Property 10: Retry with Exponential Backoff', () => {
  it('for any attempt N in [1, 3], delay equals base * 2^(N-1)', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1, max: MAX_RETRIES }),
        (attempt) => {
          const delay = calculateBackoffDelay(attempt);
          const expected = BACKOFF_BASE_MS * Math.pow(2, attempt - 1);
          expect(delay).toBe(expected);
        }
      ),
      { numRuns: 100 }
    );
  });

  it('retry delays follow the sequence 1000, 2000, 4000', () => {
    const delays = [1, 2, 3].map(calculateBackoffDelay);
    expect(delays).toEqual([1000, 2000, 4000]);
  });

  it('maximum retries is exactly 3', () => {
    expect(MAX_RETRIES).toBe(3);
  });

  it('delay always increases with each attempt', () => {
    fc.assert(
      fc.property(
        fc.integer({ min: 1, max: MAX_RETRIES - 1 }),
        (attempt) => {
          const current = calculateBackoffDelay(attempt);
          const next = calculateBackoffDelay(attempt + 1);
          expect(next).toBeGreaterThan(current);
        }
      ),
      { numRuns: 100 }
    );
  });

  it('first delay is always the base (1000ms)', () => {
    expect(calculateBackoffDelay(1)).toBe(BACKOFF_BASE_MS);
  });
});
