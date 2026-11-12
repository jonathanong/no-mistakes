import type { PoolClient } from 'pg';
import { from as copyFrom } from 'custom-copy-streams';
export function run(client: PoolClient) {
  client.query(copyFrom('/* named */ COPY users FROM STDIN'));
}
