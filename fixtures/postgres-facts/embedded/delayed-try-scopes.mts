import type { PoolClient } from 'pg';
// An imported type is not an executor unless it is configured.
import type { OtherClient } from 'pg';
declare const pool: { connect(): Promise<PoolClient> };
declare const other: OtherClient;

export async function scopes() {
  let client: PoolClient | undefined;
  try {
    client = await pool.connect();
    const box = { value: other };
    box.value = other;
    // These same-named writes belong to inner lexical bindings only.
    try { throw other; } catch (client) { client.query('SELECT catch_shadow'); }
    for (let client of [other]) { client.query('SELECT for_of_shadow'); }
    for (let client in {}) { client.query('SELECT for_in_shadow'); }
    { let client = other; client = other; client.query('SELECT block_shadow'); }
    await client.query('/* retained_outer */ SELECT 1');
  } finally { client?.release(); }
}

export async function unconfigured() {
  let client: OtherClient | undefined;
  client = other;
  client.query('SELECT unconfigured_type');
}
export async function shadowed<PoolClient>() {
  let client: PoolClient | undefined;
  client = other;
  client.query('SELECT shadowed_type');
}
export async function untyped() {
  let client;
  client = other;
  client.query('SELECT no_annotation');
}
export async function assignmentOperator() {
  let client: PoolClient | undefined;
  client ??= await pool.connect();
  client.query('SELECT unsupported_assignment');
}
