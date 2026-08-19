/**
 * Measure the JS shipped by the prerendered home page and gate it against
 * the recorded baseline.
 *
 * Sums every JS file the page can load:
 *  - <astro-island> component-url / renderer-url attributes (Astro islands)
 *  - classic <script src> and <link rel="modulepreload"> references
 *
 * Usage: node scripts/measure-home-js.mjs
 * Exit: 0 when total < baseline.homeJsBytes AND total <= baseline.targetBytes
 */
import { readFileSync, existsSync, statSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const outDir = process.env.OUT_DIR || join(root, '.vercel', 'output', 'static');
const htmlPath = join(outDir, 'index.html');
const baselinePath = join(root, 'perf-baseline.json');

if (!existsSync(htmlPath)) {
  console.error(`Home page not found: ${htmlPath}`);
  process.exit(1);
}
if (!existsSync(baselinePath)) {
  console.error(`Baseline not found: ${baselinePath}`);
  process.exit(1);
}

const html = readFileSync(htmlPath, 'utf8');

const refs = new Set();
for (const m of html.matchAll(/(?:component-url|renderer-url)="([^"]+)"/g)) refs.add(m[1]);
for (const m of html.matchAll(/<script[^>]+src="([^"]+)"/g)) refs.add(m[1]);
for (const m of html.matchAll(/<link rel="modulepreload" href="([^"]+)"/g)) refs.add(m[1]);

let total = 0;
const files = [];
for (const ref of refs) {
  const file = join(outDir, ref.replace(/^\//, ''));
  if (!existsSync(file)) {
    console.error(`Referenced JS missing from build: ${ref}`);
    process.exit(1);
  }
  const size = statSync(file).size;
  total += size;
  files.push({ ref, size });
}

const baseline = JSON.parse(readFileSync(baselinePath, 'utf8'));
const ratio = total / baseline.homeJsBytes;
const pass = total < baseline.homeJsBytes && total <= baseline.targetBytes;

console.log(
  JSON.stringify(
    {
      total,
      files,
      baseline: baseline.homeJsBytes,
      target: baseline.targetBytes,
      ratio: Number(ratio.toFixed(3)),
      reductionPct: Number(((1 - ratio) * 100).toFixed(1)),
      pass,
    },
    null,
    2,
  ),
);

process.exit(pass ? 0 : 1);