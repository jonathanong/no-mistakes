// no-mistakes-disable-file postgres-require-query-annotation
import type { PoolClient } from '@example/db';
export function disabled(client: PoolClient) { client.query('/* disabled */ SELECT 5'); }
