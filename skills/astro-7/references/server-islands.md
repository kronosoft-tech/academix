# Server Islands

Source: https://docs.astro.build/en/guides/server-islands/

## Overview

Server islands allow you to on-demand render dynamic or personalized "islands" individually, without sacrificing the performance of the rest of the page. The main content renders immediately while server islands load independently in parallel.

## Requirements

- An adapter must be installed (Node.js, Netlify, Vercel, Cloudflare)
- The component must be an Astro component (`.astro` file)

## Basic Usage

Add `server:defer` to any Astro component to turn it into a server island:

```astro
---
import Avatar from '../components/Avatar.astro';
---
<Avatar server:defer />
```

The component renders on-demand via a separate request after the page loads.

## Server Island Component

Inside the deferred component, you can do anything available in on-demand rendered pages — fetch data, access cookies, call APIs:

```astro
---
import { getUserAvatar } from '../sessions';

const userSession = Astro.cookies.get('session');
const avatarURL = await getUserAvatar(userSession);
---
<img alt="User avatar" src={avatarURL} />
```

## Fallback Content

Provide placeholder content using `slot="fallback"`:

```astro
---
import Avatar from '../components/Avatar.astro';
import GenericAvatar from '../components/GenericAvatar.astro';
---
<Avatar server:defer>
  <GenericAvatar slot="fallback" />
</Avatar>
```

Fallback examples: generic avatars, skeleton UI, loading spinners, placeholder messages.

## Passing Props

Props must be serializable. Supported types:
- `string`, `number`, `boolean`
- Plain objects, `Array`, `Map`, `Set`
- `RegExp`, `Date`, `BigInt`, `URL`
- `Uint8Array`, `Uint16Array`, `Uint32Array`
- `Infinity`

**NOT supported**: functions, objects with circular references.

## How It Works

1. At build time, the component content is replaced with a small script
2. The script fetches the component from a special endpoint at runtime
3. Each island loads independently — a slow island doesn't block others
4. Props are encrypted and passed as a query string (GET request)

## Caching

- Data is fetched via `GET` with props as an encrypted query string
- Standard `Cache-Control` HTTP headers can be used for caching
- If props cause URL to exceed 2048 bytes, Astro sends a `POST` instead (not cacheable)
- Keep props minimal to stay under the URL limit

## Accessing Page URL

`Astro.url` inside a server island returns `/_server-islands/ComponentName`, NOT the page URL.

To get the actual page URL, use the `Referer` header:

```astro
---
const referer = Astro.request.headers.get("Referer");
if (!referer) throw new Error("Referer header is missing");
const url = new URL(referer);
const productId = url.searchParams.get("product");
---
```

## Encryption Key (Production)

For rolling deployments, multi-region hosting, or CDN-cached pages with server islands, generate a stable encryption key:

```bash
astro create-key
```

Set the value as the `ASTRO_KEY` environment variable in your build environment.

## Use Cases

- User avatars/profile info on static pages
- Personalized promotions on e-commerce product pages
- Dynamic reviews or comments
- Auth-dependent navigation
- Any personalized content on an otherwise cacheable page

Content was rephrased for compliance with licensing restrictions.
