import SQL from 'sql-template-strings';
import { read } from '@example/db';

function make() { return SQL`SELECT 1 OFFSET 2`; }
function opaque(value: unknown) { return value; }
export const exported = SQL`SELECT 1 OFFSET 7`;

export function localOperands() {
  const statement = make();
  read(statement); // COMPLETE: exact const helper result.
  const updates = { a: SQL`SELECT 1 OFFSET 1`, b: SQL`SELECT 1` };
  read(updates['a']); // COMPLETE: own literal key.
  const selectedUpdates = { b: SQL`SELECT 1` };
  const selected = selectedUpdates['b'];
  read(selected); // COMPLETE: member read through another const.

  const escaped = { a: SQL`SELECT 1` };
  const alias = escaped;
  opaque(alias);
  read(escaped.a); // RESIDUAL: dependent const escapes to an opaque call.

  const changed = { a: SQL`SELECT 1` };
  changed.a = SQL`SELECT 1 OFFSET 3`;
  read(changed.a); // RESIDUAL: property write.
  read(exported); // RESIDUAL: exported module binding.
  read(later); // RESIDUAL: declaration follows the sink.
  const later = SQL`SELECT 1`;
  // The circular initializers are intentionally invalid at runtime; neither is a value proof.
  const cycleA = cycleB;
  const cycleB = cycleA;
  read(cycleA); // RESIDUAL: recursive const initializers.
}
