/**
 * Markdown heading extraction + GitHub-style slugger (blog R4/R5).
 *
 * The Astro 7 Sätteri markdown processor natively assigns `id` attributes to
 * h1-h6 headings using the `github-slugger` algorithm. This module replicates
 * that algorithm (including the `-1`, `-2`, ... duplicate suffix) so TOC jump
 * links built from `entry.body` match the rendered heading anchors exactly.
 */

export interface Heading {
  depth: number;
  text: string;
  id: string;
}

const HEADING_RE = /^(#{1,6})\s+(.+)$/gm;

/**
 * GitHub-style slug: lowercase, strip punctuation (keeping letters, numbers,
 * spaces, hyphens), collapse whitespace runs to single hyphens.
 */
export function slugify(text: string): string {
  return text
    .toLowerCase()
    .trim()
    .replace(/[^\p{L}\p{N}\s-]/gu, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-');
}

/**
 * Extract h2/h3 headings from raw markdown source with GitHub-style ids.
 * Duplicate headings get numeric suffixes (-1, -2, ...) to match the
 * github-slugger behavior used by Sätteri when rendering.
 */
export function extractHeadings(markdown: string): Heading[] {
  const headings: Heading[] = [];
  const seen = new Map<string, number>();

  for (const match of markdown.matchAll(HEADING_RE)) {
    const depth = match[1].length;
    if (depth < 2 || depth > 3) continue;

    const text = match[2].trim();
    if (!text) continue;

    const base = slugify(text);
    const occurrence = (seen.get(base) ?? 0) + 1;
    seen.set(base, occurrence);
    const id = occurrence === 1 ? base : `${base}-${occurrence - 1}`;

    headings.push({ depth, text, id });
  }

  return headings;
}