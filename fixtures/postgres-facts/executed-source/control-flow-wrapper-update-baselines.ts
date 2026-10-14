import { read, sql } from '@example/db';
function appendDelete(target: unknown) { target.append(sql`; DELETE FROM records`); return 1; }
function noOp() { return 0; }
export function wrappedBuilderAlternatives(flag: boolean) {
  const query = sql`SELECT 1`;
  // Existing wrapper aliases must preserve both canonical update alternatives.
  const promise = (async () => query)();
  const object = { ['builder']: query };
  const aggregate = flag && query;
  read(query, flag ? appendDelete(query) : noOp());
}
export function exposedWrapperStaysUncertain(flag: boolean) {
  const query = sql`SELECT 1`;
  const values = [query];
  opaqueConsumer(values);
  const possible = values[0];
  // Baseline traversal must not turn uncertain exposed aliases into SQL proof.
  read(query, flag ? noOp() : noOp());
}
