import type { PoolClient } from 'pg';
export function existing(client: PoolClient) {
  return client.query('/* parameter */ SELECT 3');
}
