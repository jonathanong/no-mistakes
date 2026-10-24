import SQL from 'sql-template-strings';
import { read, type QueryExecutor } from '@example/db';
export function deadAnd(flag: boolean) {
  if (flag && false) return SQL`SELECT 1 OFFSET 1`;
  return SQL`SELECT 1`;
}
export function deadOr(flag: boolean) {
  if (flag || true) return SQL`SELECT 1`;
  return SQL`SELECT 1 OFFSET 2`;
}
export function live(flag: boolean) {
  if (flag && true) return SQL`SELECT 1 OFFSET 3`;
  return SQL`SELECT 1`;
}
export function inherited() {
  const options = {};
  if (options.constructor) return SQL`SELECT 1 OFFSET 4`;
  return SQL`SELECT 1`;
}
export function ownField() {
  const options = { constructor: false };
  if (options.constructor) return SQL`SELECT 1 OFFSET 5`;
  return SQL`SELECT 1`;
}
export function sameExecutor(a: QueryExecutor, b: QueryExecutor) {
  if (a === b) return a(SQL`SELECT 1 OFFSET 6`);
  return b(SQL`SELECT 1`);
}
export function memberCarrier(query: QueryExecutor) { return query.other; }
export function logicalCarrier(query: QueryExecutor) { return query.other || null; }
export function consumedMember(query: QueryExecutor) { return read(SQL`SELECT ${query.other}`); }
export async function unsupportedAfterSink(query: QueryExecutor) {
  const [row] = await query(SQL`SELECT 1 OFFSET 7`);
  return row;
}
export function precision(query: QueryExecutor, flag: boolean) {
  read(deadAnd(flag)); // CLEAR_DEAD_AND
  read(deadOr(flag)); // CLEAR_DEAD_OR
  read(live(flag)); // LIVE_OFFSET
  read(inherited()); // UNKNOWN_INHERITED
  read(ownField()); // CLEAR_OWN_FIELD
  sameExecutor(query, query); // SAME_EXECUTOR_OFFSET
  memberCarrier(query); // UNKNOWN_MEMBER_CARRIER
  logicalCarrier(query); // UNKNOWN_LOGICAL_CARRIER
  consumedMember(query); // CLEAR_CONSUMED_MEMBER
  unsupportedAfterSink(query); // KNOWN_OFFSET_AND_OPAQUE_BINDING
}
