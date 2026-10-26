import SQL from 'sql-template-strings';
import { read, type QueryExecutor, type QueryOptions } from '@example/db';
import { forward, executeOptions, fragment, offsetFragment, qualifier, fixedFragment, escaped, dynamicFragment, escapedBuilder, recursive } from './helpers.mjs';
export function typedTransport(query: QueryExecutor, options: QueryOptions, id: string) {
  forward(query, id); // CLEAR
  executeOptions(options, id); // CLEAR
}
export function builders(id: string, enabled: boolean) {
  read(SQL`SELECT ${id} AS id`.append(fragment({ enabled, value: id }))); // CLEAR
  read(SQL`SELECT ${id} AS id`.append(fragment())); // CLEAR
  read(SQL`SELECT ${id} AS id WHERE `.append(qualifier('item'))); // CLEAR
  read(SQL`SELECT ${id} AS id WHERE 1 = 1`.append(fixedFragment())); // CLEAR
  read(SQL`SELECT ${id} AS id`.append(offsetFragment({ offset: true }))); // OFFSET
}
export function unknowns(query: QueryExecutor, text: string) {
  escaped(query, text); // UNKNOWN
  recursive(query); // UNKNOWN
  read(SQL`SELECT 1`.append(dynamicFragment(text))); // UNKNOWN
  read(SQL`SELECT 1`.append(escapedBuilder())); // UNKNOWN
}
