import type { PoolClient } from 'pg';
import { to as copyTo } from 'pg-copy-streams';
export function run(client: PoolClient) {
  client.query(copyTo('/* named */ COPY users TO STDOUT'));
}
