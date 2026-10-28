import SQL from 'sql-template-strings';
import { read } from '@example/db';

export function priorExecutorUses() {
  const whole = { sql: SQL`SELECT 1` };
  read(whole); // setup: the executor receives the entire mutable map.
  read(whole.sql); // CHECK: prior whole-map escape must remain residual.

  const aliasMap = { sql: SQL`SELECT 2` };
  const alias = aliasMap;
  read(alias); // setup: a const alias exposes the mutable map.
  read(aliasMap.sql); // CHECK: prior alias escape must remain residual.

  const boundMap = { sql: SQL`SELECT 3` };
  read('SELECT 3', [boundMap.sql]); // setup: a later argument exposes the builder.
  read(boundMap.sql); // CHECK: prior nested builder escape must remain residual.

  const reused = { sql: SQL`SELECT 4` };
  read(reused.sql); // setup: the same mutable builder is supplied at two sinks.
  read(reused.sql); // CHECK: a second sink cannot certify an unchanged builder.

  const getterMap = { sql: SQL`SELECT 5` };
  const unknownObject: { member: string } = globalThis as unknown as { member: string };
  read(getterMap.sql, [getterMap.sql, unknownObject.member]); // CHECK: another member may be a getter.
}
