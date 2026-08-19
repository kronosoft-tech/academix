import type { APIRoute } from 'astro';
import { getCollection } from 'astro:content';
import { SITE_URL } from '../lib/site';

export const prerender = true;

export const GET: APIRoute = async () => {
  const posts = await getCollection('blog');
  const published = posts.filter((post) => !post.data.draft);
  const ordered = [...published].sort((a, b) => {
    if (a.data.isPillar !== b.data.isPillar) return a.data.isPillar ? -1 : 1;
    return b.data.pubDate.getTime() - a.data.pubDate.getTime();
  });

  const lines = [
    '# Academix',
    '',
    '> Sistema de gestión académica para academias en Latinoamérica: estudiantes, cursos, pagos, asistencia y reportes en un solo lugar.',
    '',
    '## Blog',
    ...ordered.map((post) => `- [${post.data.title}](${SITE_URL}/blog/${post.id})`),
    '',
  ];

  return new Response(lines.join('\n'), {
    headers: { 'Content-Type': 'text/plain; charset=utf-8' },
  });
};