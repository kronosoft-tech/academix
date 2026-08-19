import { describe, expect, it } from 'vitest';
import { blogEntrySchema } from '../lib/blog-schema';

const validEntry = {
  title: 'Qué es un SGA',
  description: 'Un sistema de gestión académica centraliza la operación.',
  pubDate: '2026-03-09',
  author: 'Equipo Academix',
  tags: ['fundamentos'],
};

describe('blogEntrySchema (blog R1/R2)', () => {
  it('accepts a valid entry', () => {
    const result = blogEntrySchema.safeParse(validEntry);
    expect(result.success).toBe(true);
  });

  it('rejects a missing pubDate', () => {
    const { pubDate: _pubDate, ...withoutDate } = validEntry;
    const result = blogEntrySchema.safeParse(withoutDate);
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues.some((issue) => issue.path.includes('pubDate'))).toBe(true);
    }
  });

  it('rejects an empty title', () => {
    const result = blogEntrySchema.safeParse({ ...validEntry, title: '' });
    expect(result.success).toBe(false);
  });

  it('rejects an invalid date string', () => {
    const result = blogEntrySchema.safeParse({ ...validEntry, pubDate: 'not-a-date' });
    expect(result.success).toBe(false);
  });

  it('coerces a string date into a Date', () => {
    const parsed = blogEntrySchema.parse(validEntry);
    expect(parsed.pubDate).toBeInstanceOf(Date);
  });

  it('defaults isPillar and draft to false', () => {
    const parsed = blogEntrySchema.parse(validEntry);
    expect(parsed.isPillar).toBe(false);
    expect(parsed.draft).toBe(false);
  });

  it('accepts an explicit pillar entry with optional fields', () => {
    const parsed = blogEntrySchema.parse({
      ...validEntry,
      isPillar: true,
      updatedDate: '2026-07-01',
      coverImage: '/og-default.png',
    });
    expect(parsed.isPillar).toBe(true);
    expect(parsed.updatedDate).toBeInstanceOf(Date);
  });

  it('rejects tags that are not strings', () => {
    const result = blogEntrySchema.safeParse({ ...validEntry, tags: ['ok', 42] });
    expect(result.success).toBe(false);
  });
});