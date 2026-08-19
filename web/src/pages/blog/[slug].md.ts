import type { APIRoute } from 'astro';
import { getCollection, type CollectionEntry } from 'astro:content';

export const prerender = true;

export async function getStaticPaths() {
  const posts = await getCollection('blog');
  return posts
    .filter((post) => !post.data.draft)
    .map((post) => ({
      params: { slug: post.id },
      props: { post },
    }));
}

export const GET: APIRoute = async ({ props }) => {
  const post = (props as { post: CollectionEntry<'blog'> }).post;
  return new Response(post.body, {
    headers: { 'Content-Type': 'text/markdown; charset=utf-8' },
  });
};