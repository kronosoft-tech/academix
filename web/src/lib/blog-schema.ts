import { z } from 'astro/zod';

/**
 * Blog entry frontmatter schema (blog R1/R2).
 * Kept in a pure module so unit tests exercise the exact schema the build
 * validates via content.config.ts.
 */
export const blogEntrySchema = z.object({
  title: z.string().min(1),
  description: z.string(),
  pubDate: z.coerce.date(),
  author: z.string(),
  tags: z.array(z.string()),
  isPillar: z.boolean().default(false),
  updatedDate: z.coerce.date().optional(),
  draft: z.boolean().default(false),
  coverImage: z.string().optional(),
});

export type BlogEntryData = z.infer<typeof blogEntrySchema>;