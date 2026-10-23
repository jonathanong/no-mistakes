import type { PoolClient } from 'pg';
declare const pool: { connect(): Promise<PoolClient> };
declare const replacement: PoolClient;

export async function shorthand() {
  let client: PoolClient | undefined;
  client = await pool.connect();
  ({ client } = { client: replacement });
  client.query('SELECT shorthand_write');
}
export async function defaultBinding() {
  let client: PoolClient | undefined;
  client = await pool.connect();
  ({ client = replacement } = {});
  client.query('SELECT default_binding_write');
}
export async function wrappedBinding() {
  let client: PoolClient | undefined;
  client = await pool.connect();
  (client as PoolClient) = replacement;
  client.query('SELECT wrapped_binding_write');
}

export async function nestedKeyWrite() {
  let client: PoolClient | undefined;
  client = await pool.connect();
  const seen = {};
  // This key really rebinds client; unlike reading client.processID, it revokes proof.
  seen[client = replacement] = true;
  client.query('SELECT nested_key_write');
}
