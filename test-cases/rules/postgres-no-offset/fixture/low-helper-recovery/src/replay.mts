import SQL from 'sql-template-strings';
import { read, type QueryExecutor, type QueryOptions } from '@example/db';
import { fragment } from './helpers.mjs';
declare const dynamic: string;
export function localBuilder(options: QueryOptions, id: string, enabled: boolean) {
  const statement = SQL`SELECT ${id} AS id`;
  statement.append(fragment({ enabled, value: id }));
  read(statement, options); // CLEAR_LOCAL_BUILDER
}
export function separateSinks(query: QueryExecutor, text: string) {
  const statement = SQL`SELECT 1 OFFSET 4`;
  read(statement); // KNOWN_NEIGHBOR
  query(text); // UNKNOWN_EXACT_TARGET
}
export function closureMutation() {
  const statement = SQL`SELECT 1`;
  function mutate() { statement.append(dynamic); }
  mutate();
  read(statement); // UNKNOWN_CLOSURE_MUTATION
}
export function unusedFormal(query: QueryExecutor, text: string) {
  read(text); // UNKNOWN_UNUSED_FORMAL
}
