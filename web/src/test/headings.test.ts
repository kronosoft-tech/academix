import { describe, it, expect } from 'vitest';
import { slugify, extractHeadings } from '../lib/headings';

describe('slugify', () => {
  it('lowercases and replaces spaces with hyphens', () => {
    expect(slugify('Qué es un SGA')).toBe('qué-es-un-sga');
  });

  it('strips punctuation and symbols', () => {
    expect(slugify('Pagos & cobros: ¿cómo?')).toBe('pagos-cobros-cómo');
  });

  it('keeps letters, numbers, spaces and hyphens', () => {
    expect(slugify('Nivel B2 — Curso 2026')).toBe('nivel-b2-curso-2026');
  });

  it('collapses multiple spaces and hyphens', () => {
    expect(slugify('  Doble   espacio  ')).toBe('doble-espacio');
  });
});

describe('extractHeadings', () => {
  const markdown = [
    '# Title only (h1, ignored)',
    '## Primera sección',
    '### Sub-sección A',
    '#### H4 ignored',
    '## Primera sección', // duplicate → -1 suffix
    '### Sub-sección B',
    '## Segunda sección',
  ].join('\n');

  it('extracts only h2 and h3 with GitHub-style ids', () => {
    const headings = extractHeadings(markdown);
    expect(headings).toEqual([
      { depth: 2, text: 'Primera sección', id: 'primera-sección' },
      { depth: 3, text: 'Sub-sección A', id: 'sub-sección-a' },
      { depth: 2, text: 'Primera sección', id: 'primera-sección-1' },
      { depth: 3, text: 'Sub-sección B', id: 'sub-sección-b' },
      { depth: 2, text: 'Segunda sección', id: 'segunda-sección' },
    ]);
  });

  it('returns an empty list for content without headings', () => {
    expect(extractHeadings('Solo texto plano.')).toEqual([]);
  });

  it('ignores setext-style false positives', () => {
    expect(extractHeadings('# no heading\n\njust text')).toEqual([]);
  });
});