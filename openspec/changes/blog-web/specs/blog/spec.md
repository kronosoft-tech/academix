# Delta for Blog

## ADDED Requirements

### Requirement: R1: Blog content collection schema and validation

The system MUST define a `blog` content collection via a `glob()` loader over `web/src/content/blog/**/*.md` with a Zod schema. Required fields: `title` (non-empty string), `description` (string), `pubDate` (date), `author` (string), `tags` (array of strings), `isPillar` (boolean, default `false`). Optional fields: `updatedDate` (date), `draft` (boolean, default `false`), `coverImage` (string). An entry violating the schema MUST fail the build.

#### Scenario: Valid post passes validation

- GIVEN a Markdown post with all required frontmatter fields
- WHEN the site builds
- THEN the entry passes Zod validation
- AND it is queryable via `getCollection('blog')`

#### Scenario: Invalid frontmatter fails the build

- GIVEN a post missing a required field such as `pubDate`
- WHEN the site builds
- THEN the build fails with a schema validation error naming the field

### Requirement: R2: Draft exclusion

The system MUST exclude posts with `draft: true` from the public listing, detail routes, and sitemap, while keeping them in the source collection for authoring.

#### Scenario: Draft post hidden from public

- GIVEN a post with `draft: true`
- WHEN `/blog` and `/blog/{slug}` are requested and the sitemap is generated
- THEN the post does not appear in the listing
- AND its slug returns 404
- AND it is absent from the sitemap

### Requirement: R3: Blog listing page

The system MUST serve a prerendered listing at `/blog` rendering all published posts ordered by `pubDate` descending, grouped by topic cluster with a link to the pillar post. The page MUST be pure Astro with zero client-side JavaScript.

#### Scenario: Published posts listed newest first

- GIVEN two or more published posts
- WHEN `/blog` is requested
- THEN a 200 HTML response lists them newest-first with the pillar linked at the top
- AND the page ships no hydration scripts

#### Scenario: Empty collection

- GIVEN zero published posts
- WHEN `/blog` is requested
- THEN a 200 HTML response renders an empty state without errors

### Requirement: R4: Blog post page

The system MUST serve prerendered detail pages at `/blog/[...slug]` via `getStaticPaths()` with `prerender = true`, rendering the Markdown `Content`, a table of contents, related posts, the lead-magnet/trial CTA, and a `<link rel="alternate" type="text/markdown">` pointing to the post's `.md` route.

#### Scenario: Known slug renders the article

- GIVEN a published post
- WHEN `/blog/{slug}` is requested
- THEN a 200 HTML response renders the article with TOC, CTA, and the markdown alternate link

#### Scenario: Unknown slug returns 404

- GIVEN a slug with no matching published post
- WHEN the route is requested
- THEN the response is 404

### Requirement: R5: Pillar page anatomy and internal linking

A post with `isPillar: true` MUST render a HubSpot-style pillar page: intro, jump-link TOC, one section per subtopic linking out to the corresponding cluster post with descriptive anchor text, and a conversion CTA. Cluster posts MUST link back to the pillar. No pillar content MAY be locked behind forms or paywalls.

#### Scenario: Pillar renders with TOC, link-outs, and CTA

- GIVEN a pillar post with subtopic sections
- WHEN `/blog/{pillar-slug}` is requested
- THEN the page renders jump-link TOC entries, subtopic link-outs, and the CTA
- AND all content is readable without submitting a form

#### Scenario: Cluster post links back to the pillar

- GIVEN a cluster post covering a pillar subtopic
- WHEN the post renders
- THEN it contains a descriptive-text link back to the pillar URL

### Requirement: R6: Conversion CTAs and lead magnet

The system MUST render a trial CTA and a downloadable lead-magnet guide CTA on the pillar and post pages. The trial CTA MUST link to the existing registration/trial flow; the guide CTA MUST link to a downloadable asset. CTAs MUST NOT gate article content.

#### Scenario: Trial CTA navigates to registration

- GIVEN a published post with a trial CTA
- WHEN the reader activates the CTA
- THEN they are taken to the registration/trial flow

#### Scenario: Lead magnet guide is downloadable

- GIVEN the pillar page with a guide CTA
- WHEN the reader activates the guide CTA
- THEN the downloadable guide asset is served

### Requirement: R7: Public routing and navigation

The system MUST treat `/blog` and all `/blog/` paths as public by adding them to the middleware `PUBLIC_ROUTES`, so unauthenticated visitors pass through. The `Base.astro` layout MUST expose a Blog link in nav and footer.

#### Scenario: Anonymous access to blog routes

- GIVEN a request with no auth cookie
- WHEN `/blog` or `/blog/{slug}` is requested
- THEN a 200 HTML response is served without redirect to login

#### Scenario: Blog link in nav

- GIVEN any page rendered with `Base.astro`
- WHEN the page is displayed
- THEN the nav contains a Blog link pointing to `/blog`

## Out of Scope (Non-Requirements)

The system MUST NOT implement a CMS/DB-backed blog, full i18n beyond i18n-ready frontmatter/routes, a landing MUI redesign, or Stripe additions as part of this capability.