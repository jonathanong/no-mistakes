import type { PoolClient, Executor } from '@example/db';
export function run(client: PoolClient, replacement: PoolClient, known: Executor, unrelated: unknown) {
  client = replacement;
  client.query('SELECT explicit_parameter');
  let local: PoolClient = client;
  local = replacement;
  local.query('SELECT explicit_local');
  let inferred = client;
  inferred = unrelated;
  inferred.query('SELECT ignored_inferred');
  function nested(client: unknown) { client.query('SELECT ignored_shadow'); }
  let declared: Executor = known;
  declared = known;
  declared.query('SELECT unresolved_declared'); // Local annotation alone is not executor proof.
  known.query('SELECT declared_executor');
}
// Nullable imported types use the same union grammar as declared executors.
export function nullable(client: undefined | PoolClient) {
  if (client) client.query('SELECT nullable_receiver');
}
// Object-pattern parameter properties use the existing declared-type grammar.
export function destructured({ client }: { client: PoolClient }) {
  client.query('SELECT destructured_receiver');
}
