import type { PoolClient } from 'pg';
declare const pool: { connect(): Promise<PoolClient> };

export async function run() {
  let client: PoolClient | undefined;
  try {
    client = await pool.connect();
    await client.query('/* named_try */ SELECT 1');
    await client.query('SELECT 2');
    // A read in another assignment target must not revoke earlier executor calls.
    const seen = {};
    seen[client.processID] = true;
    client.application_name = 'audit';
    seen[client.processID]++;
    ({ [client.processID]: seen.found } = seen);
    ({ found: seen.found = client.processID } = seen);
    await client.query('/* after_reads */ SELECT 3');
  } finally {
    client?.release();
  }
}
