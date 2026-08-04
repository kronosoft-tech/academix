import Groq from 'groq-sdk';
import type { AIProvider, ChatMessage } from './types';

export class GroqProvider implements AIProvider {
  name = 'groq';
  private client: Groq;
  private model: string;

  constructor() {
    const apiKey = import.meta.env.GROQ_API_KEY;
    if (!apiKey) {
      throw new Error('GROQ_API_KEY environment variable is not set');
    }
    this.client = new Groq({ apiKey });
    this.model = 'llama-3.3-70b-versatile';
  }

  async *chat(messages: ChatMessage[]): AsyncGenerator<string> {
    const stream = await this.client.chat.completions.create({
      model: this.model,
      messages: messages.map((m) => ({ role: m.role, content: m.content })),
      stream: true,
    });

    for await (const chunk of stream) {
      const content = chunk.choices[0]?.delta?.content;
      if (content) {
        yield content;
      }
    }
  }
}
