import type { PoolClient } from 'pg';
import * as copies from 'pg-copy-streams';
export function run(client: PoolClient) {
  client.query(copies.from('/* named */ COPY users FROM STDIN'));
}
