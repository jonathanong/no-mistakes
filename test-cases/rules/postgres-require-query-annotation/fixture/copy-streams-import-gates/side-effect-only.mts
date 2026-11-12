import type { PoolClient } from 'pg';
import 'pg-copy-streams';
declare const from: (sql: string) => unknown;
export function run(client: PoolClient) {
  client.query(from('/* named */ COPY users FROM STDIN'));
}
