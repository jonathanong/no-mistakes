import type { Executor, PoolClient as Client } from '@example/db';
import { PoolClient } from '@example/db/client';
import type { PoolClient as Other } from '@example/db-other';

export function run(declared: Executor, omitted: Client, unrelated: Other) {
  declared.query('SELECT 1');
  omitted.query('SELECT 2');
  omitted['query']('SELECT 3');
  omitted.execute('SELECT 4');
  omitted.other('SELECT 5');
  unrelated.query('SELECT 6');
  // An untyped shadow must not inherit the outer receiver's imported type.
  function nested(omitted: unknown) { omitted.query('SELECT 7'); }
  { const omitted = unrelated; omitted.query('SELECT 8'); }
  omitted.query('SELECT 9');
}
const client = new PoolClient();
client.query('SELECT 10');
const annotated: Client = acquire();
annotated.query('SELECT 11');
const arrow = (receiver: Client) => receiver.query('SELECT 12');
export function ignored({ receiver }: { receiver: Other }, untyped, primitive: string) { // Unconfigured-module negative control.
  receiver.query('SELECT ignored');
  untyped.query('SELECT ignored');
  primitive.query('SELECT ignored');
}
const plain = 1;
const arbitrary = new Unknown();
const holder = new ns.Unknown();
const { destructured } = acquire();
client[dynamic]('SELECT ignored');
client.query.call(null, 'SELECT ignored');
query('SELECT ignored');
acquire().query('SELECT ignored');
const alias = client;
alias.query('SELECT 13');
// Reassignment and constructor shadows must not inherit stale import provenance.
let mutable = client;
mutable = unrelated;
mutable.query('SELECT ignored');
function constructorShadow(PoolClient: unknown) {
  const local = new PoolClient();
  local.query('SELECT ignored');
}
Client.query('SELECT ignored'); // Type-only imports have no runtime receiver.
// Explicit declared types take precedence over inferred alias identities.
const explicitlyDeclared: Executor = client;
explicitlyDeclared.query('SELECT ignored');
