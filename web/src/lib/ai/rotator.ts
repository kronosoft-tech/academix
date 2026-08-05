import type { AIProvider, ChatMessage } from './types';

export class ProviderRotator {
  private providers: AIProvider[];
  private counter = 0;

  constructor(providers: AIProvider[]) {
    if (providers.length === 0) {
      throw new Error('At least one AI provider is required');
    }
    this.providers = providers;
  }

  async *chat(messages: ChatMessage[]): AsyncGenerator<string> {
    const startIndex = this.counter % this.providers.length;
    this.counter++;

    let lastError: Error | null = null;

    for (let attempt = 0; attempt < this.providers.length; attempt++) {
      const index = (startIndex + attempt) % this.providers.length;
      const provider = this.providers[index];

      try {
        const stream = provider.chat(messages);
        // Attempt to get the first chunk to verify the provider works
        const first = await stream.next();
        if (!first.done && first.value) {
          yield first.value;
        }
        // Yield remaining chunks
        for await (const chunk of stream) {
          yield chunk;
        }
        return;
      } catch (err) {
        lastError = err instanceof Error ? err : new Error(String(err));
        // Continue to next provider
      }
    }

    throw new Error(
      `Todos los proveedores de IA no están disponibles. Por favor crea un ticket de soporte. (${lastError?.message})`
    );
  }
}
