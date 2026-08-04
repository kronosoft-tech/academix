import { createClient, type Client } from '@libsql/client';

let client: Client | null = null;

export function getDb(): Client {
  if (!client) {
    const url = import.meta.env.TURSO_URL;
    const authToken = import.meta.env.TURSO_AUTH_TOKEN;

    if (!url) {
      throw new Error('TURSO_URL environment variable is not set');
    }

    client = createClient({
      url,
      authToken,
    });
  }
  return client;
}

export const db = new Proxy({} as Client, {
  get(_target, prop) {
    return (getDb() as Record<string | symbol, unknown>)[prop];
  },
});
