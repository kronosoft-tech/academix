export const prerender = false;

import type { APIRoute } from 'astro';
import { z } from 'astro/zod';
import { verifyToken, getAuthCookie } from '../../lib/auth';
import { ProviderRotator } from '../../lib/ai/rotator';
import { GroqProvider } from '../../lib/ai/groq';
import { CerebrasProvider } from '../../lib/ai/cerebras';
import { db } from '../../lib/db';
import type { ChatMessage } from '../../lib/ai/types';

const messagesSchema = z.object({
  messages: z.array(
    z.object({
      role: z.enum(['user', 'assistant', 'system']),
      content: z.string().min(1),
    })
  ),
});

const SYSTEM_PROMPT: ChatMessage = {
  role: 'system',
  content:
    'Eres el asistente técnico de Academix, una aplicación de escritorio para gestión académica. Ayudas a los usuarios con problemas técnicos de la aplicación, descarga, instalación, y uso general del sistema. Responde siempre en español y de forma concisa.',
};

function createRotator(): ProviderRotator {
  const providers = [];

  try {
    providers.push(new GroqProvider());
  } catch {
    // Groq not configured — skip
  }

  try {
    providers.push(new CerebrasProvider());
  } catch {
    // Cerebras not configured — skip
  }

  if (providers.length === 0) {
    throw new Error('No AI providers configured');
  }

  return new ProviderRotator(providers);
}

export const POST: APIRoute = async ({ request, cookies }) => {
  // Authenticate user
  const token = getAuthCookie(cookies);
  if (!token) {
    return new Response(JSON.stringify({ error: 'No autorizado' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  let userId: string;
  try {
    const payload = await verifyToken(token);
    if (payload.type !== 'customer') {
      return new Response(JSON.stringify({ error: 'No autorizado' }), {
        status: 403,
        headers: { 'Content-Type': 'application/json' },
      });
    }
    userId = payload.sub;
  } catch {
    return new Response(JSON.stringify({ error: 'Token inválido' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  // Parse and validate body
  let body;
  try {
    body = await request.json();
  } catch {
    return new Response(JSON.stringify({ error: 'JSON inválido' }), {
      status: 400,
      headers: { 'Content-Type': 'application/json' },
    });
  }

  const parsed = messagesSchema.safeParse(body);
  if (!parsed.success) {
    return new Response(
      JSON.stringify({ error: 'Formato de mensajes inválido', details: parsed.error.flatten() }),
      { status: 400, headers: { 'Content-Type': 'application/json' } }
    );
  }

  const userMessages = parsed.data.messages;
  const messagesWithSystem: ChatMessage[] = [SYSTEM_PROMPT, ...userMessages];

  // Create SSE stream
  const encoder = new TextEncoder();
  let fullResponse = '';
  let providerUsed = '';

  const stream = new ReadableStream({
    async start(controller) {
      try {
        const rotator = createRotator();
        providerUsed = 'rotator';

        for await (const chunk of rotator.chat(messagesWithSystem)) {
          fullResponse += chunk;
          controller.enqueue(encoder.encode(`data: ${JSON.stringify(chunk)}\n\n`));
        }

        controller.enqueue(encoder.encode('data: [DONE]\n\n'));
      } catch (err) {
        const message = err instanceof Error ? err.message : 'Error desconocido';
        controller.enqueue(
          encoder.encode(
            `data: ${JSON.stringify({ error: 'Chat de soporte temporalmente no disponible. Por favor crea un ticket.' })}\n\n`
          )
        );
      } finally {
        controller.close();

        // Save conversation to DB (best-effort, non-blocking)
        if (fullResponse) {
          const allMessages = [
            ...userMessages,
            { role: 'assistant' as const, content: fullResponse },
          ];
          saveConversation(userId, providerUsed, allMessages).catch(() => {
            // Silently ignore save errors
          });
        }
      }
    },
  });

  return new Response(stream, {
    headers: {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      Connection: 'keep-alive',
    },
  });
};

async function saveConversation(
  userId: string,
  provider: string,
  messages: ChatMessage[]
): Promise<void> {
  const id = crypto.randomUUID();
  const now = new Date().toISOString();

  await db.execute({
    sql: `INSERT INTO ai_conversations (id, user_id, provider, model, messages_json, created_at)
          VALUES (?, ?, ?, ?, ?, ?)`,
    args: [id, userId, provider, 'auto', JSON.stringify(messages), now],
  });
}
