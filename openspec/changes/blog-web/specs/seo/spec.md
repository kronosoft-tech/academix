# Delta for SEO

## ADDED Requirements

### Requirement: R1: SEO metadata on public pages

The system MUST provide a `Seo.astro` component emitting, for every public page: canonical URL, robots meta (`index,follow,max-image-preview:large`), Open Graph (`og:title`, `og:description`, `og:type`, `og:image`, `og:url`, `og:locale` = `es_ES`), Twitter card, and JSON-LD (`WebSite`/`Organization` on site pages; `BlogPosting` on posts). All public pages via `Base.astro` and `index.astro` MUST consume it. Canonical and OG URLs MUST derive from `SITE_URL` plus `Astro.url.pathname`.

#### Scenario: Home page metadata

- GIVEN `/` is requested
- WHEN the page renders
- THEN the head contains canonical, robots, OG, Twitter, and JSON-LD entries with absolute URLs under `SITE_URL`

#### Scenario: Post structured data

- GIVEN a published post page
- WHEN the page renders
- THEN the JSON-LD `BlogPosting` block includes title, description, `pubDate`, author, and the canonical URL

### Requirement: R2: Site config and sitemap

The system MUST set `site` in `astro.config.mjs` from `SITE_URL` and integrate `@astrojs/sitemap`. The generated sitemap MUST list every prerendered page (blog included) with absolute URLs under `site`. A build with the sitemap integration and no `site` MUST fail.

#### Scenario: Sitemap covers prerendered pages

- GIVEN `SITE_URL` configured
- WHEN the site builds
- THEN `sitemap-index.xml` lists all prerendered routes as absolute URLs under `SITE_URL`, including `/blog` and all post slugs

#### Scenario: Missing site fails the build

- GIVEN no `site`/`SITE_URL` value
- WHEN the build runs with the sitemap integration
- THEN the build fails instead of emitting relative URLs

### Requirement: R3: robots.txt

The system MUST serve `robots.txt` from `web/public/` allowing all user agents including `GPTBot`, `ClaudeBot`, and `PerplexityBot`; disallowing `/api/`, `/dashboard`, and `/admin`; and pointing `Sitemap:` at the absolute sitemap URL under `SITE_URL`.

#### Scenario: Default fetch

- GIVEN `/robots.txt` is requested
- THEN a 200 `text/plain` response contains `Allow: /`, disallow rules for `/api/`, `/dashboard`, `/admin`, and the `Sitemap:` pointer

#### Scenario: AI crawler not blocked

- GIVEN a request presenting user-agent `GPTBot`
- THEN robots.txt permits crawling of public pages

### Requirement: R4: LLM discoverability

The system MUST serve `llms.txt` (H1 + blockquote + H2 link sections per llmstxt.org) and a clean Markdown route for each blog post at `{slug}.md` serving the same source content with `Content-Type: text/markdown`. Post heads MUST include `<link rel="alternate" type="text/markdown">`. The system MUST NOT serve different content based on User-Agent.

#### Scenario: llms.txt served

- GIVEN `/llms.txt` is requested
- THEN a 200 `text/plain` response follows the llmstxt.org format with links to the pillar and posts

#### Scenario: Markdown route mirrors the post

- GIVEN `/blog/{slug}.md` is requested for a published post
- THEN a 200 response with `Content-Type: text/markdown` returns the post's source Markdown identical to the collection entry

#### Scenario: No cloaking

- GIVEN the same URL requested with a browser User-Agent and with an AI crawler User-Agent
- THEN both responses are identical HTML

### Requirement: R5: Typography

The system MUST enable `@tailwindcss/typography` so `prose`-classed content (blog articles, FAQ, tutorials) renders with readable article styling. Existing prose-classed pages MUST keep their current layout.

#### Scenario: Styled article content

- GIVEN a blog post rendered with prose classes
- WHEN the article is displayed
- THEN headings, paragraphs, lists, and links render with typography styles

#### Scenario: No regression on existing prose pages

- GIVEN the existing FAQ and tutorial pages
- WHEN they render after typography is enabled
- THEN no layout breakage appears on visual regression check

### Requirement: R6: Home page performance

The system MUST NOT ship the entire landing page as a single eagerly-hydrated island. Interactive sections on `index.astro` MUST be split and hydrated with `client:visible` or lighter directives, reducing shipped JS, while preserving the current visual design.

#### Scenario: Lazy-hydrated home sections

- GIVEN `/` is requested
- WHEN the page loads
- THEN below-the-fold interactive sections hydrate only when scrolled into view
- AND the initial JS payload is smaller than the single-island baseline
- AND the visual design is unchanged

### Requirement: R7: SITE_URL environment behavior

The system MUST derive `SITE_URL` from environment configuration (the current Vercel deployment domain) and MUST NOT hardcode a guessed domain. The system MUST refresh the stale `SITE_URL` line in `web/src/.env.example`. Local development MAY fall back to `http://localhost:4321` when `SITE_URL` is unset.

#### Scenario: Production URLs use SITE_URL

- GIVEN `SITE_URL` set to the real Vercel domain
- WHEN the site builds and serves
- THEN canonical, OG, sitemap, and robots `Sitemap:` URLs all use that domain

#### Scenario: Missing SITE_URL in development

- GIVEN `SITE_URL` unset in local dev
- WHEN a canonical URL is rendered
- THEN the `localhost:4321` fallback is used without crashing

## Out of Scope (Non-Requirements)

The system MUST NOT implement `llms-full.txt`, `Accept: text/markdown` content negotiation, UA-sniffing/cloaking, or full i18n as part of this capability.