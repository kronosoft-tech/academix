import { describe, it, expect, vi } from 'vitest';
import { ProviderRotator } from '../lib/ai/rotator';
import type { AIProvider, ChatMessage } from '../lib/ai/types';

function createMockProvider(name: string, response: string): AIProvider {
  return {
    name,
    async *chat(_messages: ChatMessage[]): AsyncGenerator<string> {
      yield response;
    },
  };
}

function createFailingProvider(name: string): AIProvider {
  return {
    name,
    async *chat(_messages: ChatMessage[]): AsyncGenerator<string> {
      throw new Error(`${name} is down`);
    },
  };
}

function createDelayedProvider(name: string, response: string, delayMs: number): AIProvider {
  return {
    name,
    async *chat(_messages: ChatMessage[]): AsyncGenerator<string> {
      await new Promise((resolve) => setTimeout(resolve, delayMs));
      yield response;
    },
  };
}

async function collectStream(gen: AsyncGenerator<string>): Promise<string> {
  let result = '';
  for await (const chunk of gen) {
    result += chunk;
  }
  return result;
}

const testMessages: ChatMessage[] = [{ role: 'user', content: 'Hello' }];

describe('ProviderRotator', () => {
  it('should throw if no providers are given', () => {
    expect(() => new ProviderRotator([])).toThrow('At least one AI provider is required');
  });

  it('should use the first provider on the first call', async () => {
    const rotator = new ProviderRotator([
      createMockProvider('providerA', 'response from A'),
      createMockProvider('providerB', 'response from B'),
    ]);

    const result = await collectStream(rotator.chat(testMessages));
    expect(result).toBe('response from A');
  });

  it('should rotate through providers in round-robin fashion', async () => {
    const rotator = new ProviderRotator([
      createMockProvider('providerA', 'A'),
      createMockProvider('providerB', 'B'),
      createMockProvider('providerC', 'C'),
    ]);

    const r1 = await collectStream(rotator.chat(testMessages));
    const r2 = await collectStream(rotator.chat(testMessages));
    const r3 = await collectStream(rotator.chat(testMessages));
    const r4 = await collectStream(rotator.chat(testMessages));

    expect(r1).toBe('A');
    expect(r2).toBe('B');
    expect(r3).toBe('C');
    expect(r4).toBe('A'); // Wraps around
  });

  it('should fallback to the next provider on error', async () => {
    const rotator = new ProviderRotator([
      createFailingProvider('brokenA'),
      createMockProvider('healthyB', 'fallback response'),
    ]);

    const result = await collectStream(rotator.chat(testMessages));
    expect(result).toBe('fallback response');
  });

  it('should skip multiple failing providers until one succeeds', async () => {
    const rotator = new ProviderRotator([
      createFailingProvider('broken1'),
      createFailingProvider('broken2'),
      createMockProvider('healthy3', 'third time lucky'),
    ]);

    const result = await collectStream(rotator.chat(testMessages));
    expect(result).toBe('third time lucky');
  });

  it('should throw when ALL providers fail', async () => {
    const rotator = new ProviderRotator([
      createFailingProvider('broken1'),
      createFailingProvider('broken2'),
    ]);

    await expect(collectStream(rotator.chat(testMessages))).rejects.toThrow(
      /no están disponibles/
    );
  });

  it('should include error details when all providers fail', async () => {
    const rotator = new ProviderRotator([
      createFailingProvider('onlyProvider'),
    ]);

    await expect(collectStream(rotator.chat(testMessages))).rejects.toThrow(
      'onlyProvider is down'
    );
  });

  it('should pass messages to the provider', async () => {
    const chatSpy = vi.fn();
    const spyProvider: AIProvider = {
      name: 'spy',
      async *chat(messages: ChatMessage[]): AsyncGenerator<string> {
        chatSpy(messages);
        yield 'ok';
      },
    };

    const rotator = new ProviderRotator([spyProvider]);
    await collectStream(rotator.chat(testMessages));

    expect(chatSpy).toHaveBeenCalledWith(testMessages);
  });

  it('should handle multi-chunk streaming responses', async () => {
    const multiChunkProvider: AIProvider = {
      name: 'chunky',
      async *chat(_messages: ChatMessage[]): AsyncGenerator<string> {
        yield 'Hello, ';
        yield 'how ';
        yield 'are you?';
      },
    };

    const rotator = new ProviderRotator([multiChunkProvider]);
    const result = await collectStream(rotator.chat(testMessages));
    expect(result).toBe('Hello, how are you?');
  });
});
