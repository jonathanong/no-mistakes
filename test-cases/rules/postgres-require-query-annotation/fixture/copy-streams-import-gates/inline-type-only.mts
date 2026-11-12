import type { PoolClient } from 'pg';
import { type from } from 'pg-copy-streams';
export function run(client: PoolClient) {
  client.query(from('/* named */ COPY users FROM STDIN'));
}
