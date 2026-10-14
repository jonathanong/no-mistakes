import { read, sql } from '@example/db';
function appendDelete(target: unknown) {
  target.append(sql`; DELETE FROM records`);
  return 1;
}
function noOp() { return 0; }
export function conditionalMutation(flag: boolean) {
  const query = sql`SELECT 1`;
  read(query, flag ? appendDelete(query) : noOp());
}
export function pureAlternatives(flag: boolean) {
  const query = sql`SELECT 1`;
  read(query, flag ? noOp() : noOp());
}
export function earlierRead(flag: boolean) {
  const query = sql`SELECT 1`;
  read(query);
  flag ? appendDelete(query) : noOp();
}
function appendEmpty(target: unknown) { target.append(sql``); return 1; }
export function unchangedMutation(flag: boolean) {
  const query = sql`SELECT 1`;
  read(query, flag ? appendEmpty(query) : noOp());
}
export function bindingReplaced(flag: boolean) {
  let query = sql`SELECT 1`;
  // The first argument retains the original object when the binding is replaced.
  read(query, flag ? (query = sql`; DELETE FROM records`) : noOp());
}
export function annotatedMutation(flag: boolean) {
  const query = sql`/* branch/read */ SELECT 1`;
  read(query, flag ? appendDelete(query) : noOp());
}
