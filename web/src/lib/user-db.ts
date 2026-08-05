import { createClient, type Client } from '@libsql/client';
import type { CustomerJwtPayload } from './auth';

/**
 * Creates a libSQL client connected to the user's individual Turso database.
 * Uses the db_url and db_token stored in the JWT after login.
 */
export function getUserDb(payload: CustomerJwtPayload): Client {
  return createClient({
    url: payload.dbUrl,
    authToken: payload.dbToken,
  });
}
