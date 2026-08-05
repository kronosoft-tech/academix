import type { AIProvider, ChatMessage } from './types';

export class CerebrasProvider implements AIProvider {
  name = 'cerebras';
  private apiKey: string;
  private model: string;
  private baseUrl = 'https://api.cerebras.ai/v1/chat/completions';

  constructor() {
    const key = import.meta.env.CEREBRAS_API_KEY;
    if (!key) {
      throw new Error('CEREBRAS_API_KEY environment variable is not set');
    }
    this.apiKey = key;
    this.model = 'llama-3.3-70b';
  }

  async *chat(messages: ChatMessage[]): AsyncGenerator<string> {
    const response = await fetch(this.baseUrl, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Authorization: `Bearer ${this.apiKey}`,
      },
      body: JSON.stringify({
        model: this.model,
        messages: messages.map((m) => ({ role: m.role, content: m.content })),
        stream: true,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Cerebras API error: ${response.status} ${errorText}`);
    }

    const reader = response.body?.getReader();
    if (!reader) {
      throw new Error('No response body from Cerebras');
    }

    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n');
      buffer = lines.pop() || '';

      for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed || !trimmed.startsWith('data: ')) continue;

        const data = trimmed.slice(6);
        if (data === '[DONE]') return;

        try {
          const parsed = JSON.parse(data);
          const content = parsed.choices?.[0]?.delta?.content;
          if (content) {
            yield content;
          }
        } catch {
          // Skip malformed JSON lines
        }
      }
    }
  }
}
