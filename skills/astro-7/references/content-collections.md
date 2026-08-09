# Content Collections

Source: https://docs.astro.build/en/guides/content-collections/

## Overview

Content collections manage sets of structured content with type safety, Zod validation, and optimized querying. Two types exist: build-time collections (fetched at build) and live collections (fetched per-request at runtime).

## Build-Time Collections

### Configuration

Define in `src/content.config.ts`:

```typescript
import { defineCollection } from 'astro:content';
import { glob, file } from 'astro/loaders';
import { z } from 'astro/zod';

const blog = defineCollection({
  loader: glob({ base: './src/content/blog', pattern: '**/*.{md,mdx}' }),
  schema: z.object({
    title: z.string(),
    description: z.string(),
    pubDate: z.coerce.date(),
    updatedDate: z.coerce.date().optional(),
    draft: z.boolean().optional(),
  }),
});

const authors = defineCollection({
  loader: file("src/data/authors.json"),
  schema: z.object({
    id: z.string(),
    name: z.string(),
    portfolio: z.url(),
  }),
});

export const collections = { blog, authors };
```

### Built-in Loaders

#### `glob()` — Multiple files in a directory

```typescript
loader: glob({ pattern: "**/*.md", base: "./src/data/blog" })
```

- Supports: Markdown, MDX, Markdoc, JSON, YAML, TOML
- Auto-generates `id` from filename (URL-friendly)
- Custom IDs via `slug` frontmatter property

#### `file()` — Single file with multiple entries

```typescript
loader: file("src/data/dogs.json")
```

- Parses JSON arrays/objects, YAML, TOML
- Requires `id` property in each entry
- Supports custom parsers for CSV, etc.

### Querying

```astro
---
import { getCollection, getEntry } from 'astro:content';

// Get all entries
const allPosts = await getCollection('blog');

// Filter entries
const publishedPosts = await getCollection('blog', ({ data }) => {
  return data.draft !== true;
});

// Get single entry
const post = await getEntry('blog', 'hello-world');

// Sort by date
const sorted = allPosts.sort(
  (a, b) => b.data.pubDate.valueOf() - a.data.pubDate.valueOf()
);
---
```

### Rendering Content

```astro
---
import { getEntry, render } from "astro:content";

const entry = await getEntry("blog", "post-1");
if (!entry) throw new Error("Entry not found");
const { Content, headings } = await render(entry);
---
<h1>{entry.data.title}</h1>
<Content />
```

### Generating Routes

#### Static (default)

```astro
---
// src/pages/blog/[...id].astro
import { getCollection, render } from 'astro:content';

export async function getStaticPaths() {
  const posts = await getCollection('blog');
  return posts.map(post => ({
    params: { id: post.id },
    props: { post },
  }));
}

const { post } = Astro.props;
const { Content } = await render(post);
---
<h1>{post.data.title}</h1>
<Content />
```

#### On-demand (SSR)

```astro
---
// src/pages/blog/[...id].astro
export const prerender = false;
import { getEntry, render } from "astro:content";

const { id } = Astro.params;
if (!id) return Astro.redirect("/404");

const post = await getEntry("blog", id);
if (!post) return Astro.redirect("/404");

const { Content } = await render(post);
---
<h1>{post.data.title}</h1>
<Content />
```

### Collection References

Reference entries from other collections:

```typescript
const blog = defineCollection({
  loader: glob({ base: "./src/content/blog", pattern: "**/*.{md,mdx}" }),
  schema: z.object({
    title: z.string(),
    author: reference("authors"),
    relatedPosts: z.array(reference("blog")),
  }),
});
```

Query referenced data:

```astro
---
import { getEntry, getEntries } from "astro:content";
const blogPost = await getEntry("blog", "my-post");
const author = await getEntry(blogPost.data.author);
const related = await getEntries(blogPost.data.relatedPosts);
---
```

## Live Collections (v7)

For data that changes frequently and must be fresh per-request.

### Configuration

Define in `src/live.config.ts`:

```typescript
import { defineLiveCollection } from 'astro:content';
import { storeLoader } from '@mystore/astro-loader';

const products = defineLiveCollection({
  loader: storeLoader({
    apiKey: process.env.STORE_API_KEY,
    endpoint: 'https://api.mystore.com/v1',
  }),
});

export const collections = { products };
```

### Querying Live Data

```astro
---
export const prerender = false;
import { getLiveCollection, getLiveEntry } from "astro:content";

const { entries, error, cacheHint } = await getLiveCollection("products");
const { entry, error: entryError } = await getLiveEntry("products", Astro.params.id);
---
```

### Error Handling

```astro
---
import { LiveEntryNotFoundError } from "astro/content/runtime";
import { getLiveEntry } from "astro:content";

const { entry, error } = await getLiveEntry("products", Astro.params.id);
if (error) {
  if (error instanceof LiveEntryNotFoundError) {
    Astro.response.status = 404;
  } else {
    return Astro.redirect("/500");
  }
}
---
```

### Caching Live Data (v7)

```astro
---
import { getLiveEntry } from 'astro:content';
const { entry, cacheHint } = await getLiveEntry('products', Astro.params.id);
if (cacheHint) Astro.cache.set(cacheHint);
Astro.cache.set({ maxAge: 300 });
---
```

### Live vs Build-Time

| Aspect | Build-time | Live |
|--------|-----------|------|
| Config file | `src/content.config.ts` | `src/live.config.ts` |
| Define function | `defineCollection()` | `defineLiveCollection()` |
| Query functions | `getCollection()`, `getEntry()` | `getLiveCollection()`, `getLiveEntry()` |
| Data freshness | Build time only | Every request |
| MDX support | Yes | No |
| Image optimization | Yes | No |
| Performance | Excellent (cached) | Network cost per request |

## Custom IDs

Override the auto-generated ID with `slug` in frontmatter:

```markdown
---
title: My Blog Post
slug: my-custom-id/supports/slashes
---
```

## Schema Best Practices

1. Always define a schema for type safety
2. Use `z.coerce.date()` for date fields
3. Use `z.url()` for URL validation
4. Mark optional fields with `.optional()`
5. Use `reference()` to link collections

Content was rephrased for compliance with licensing restrictions.
