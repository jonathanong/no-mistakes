import type { PoolClient } from 'db-lib';
declare const id: number;
export async function objectLiteral() {
  let client: PoolClient | undefined;
  client = ({ query: async () => undefined, release() {} }) as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function satisfiesObject() {
  let client: PoolClient | undefined;
  client = (({ query: async () => undefined, release() {} } satisfies PoolClient)) as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function arrayLiteral() {
  let client: PoolClient | undefined;
  client = ([]) as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function nullLiteral() {
  let client: PoolClient | undefined;
  client = (null) as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function booleanLiteral() {
  let client: PoolClient | undefined;
  client = (false) as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function numberLiteral() {
  let client: PoolClient | undefined;
  client = (1) as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function stringLiteral() {
  let client: PoolClient | undefined;
  client = ('fake') as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function bigintLiteral() {
  let client: PoolClient | undefined;
  client = (1n) as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function regexpLiteral() {
  let client: PoolClient | undefined;
  client = (/fake/) as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function templateLiteral() {
  let client: PoolClient | undefined;
  client = (`fake`) as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function classLiteral() {
  let client: PoolClient | undefined;
  client = ((class {})) as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
export async function awaitedObject() {
  let client: PoolClient | undefined;
  client = (await ({ query: async () => undefined, release() {} } as PoolClient)) as unknown as PoolClient;
  await client.query('UPDATE items SET a=1 WHERE id=$1', [id]);
}
