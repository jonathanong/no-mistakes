import { read } from '@example/db';
import type { QueryOptions, QueryExecutor } from '@example/db';
import SQL from 'sql-template-strings';
declare const opaque: (value: unknown) => unknown;
declare const dynamicSql: string;
export function binds(options: QueryOptions, query: QueryExecutor) {
  read(SQL`SELECT ${options} AS value`); // BIND_CLEAR
  read(SQL`SELECT ${options.id} AS id`, options); // BIND_CLEAR
  read(SQL`SELECT ${query} AS value`); // BIND_CLEAR
}
export function bindUnknown(query: QueryExecutor) {
  read(SQL`SELECT ${opaque(query)} AS value`); // BIND_ESCAPE: opaque call owns this use.
  read(SQL`SELECT ${() => query} AS value`); // BIND_ESCAPE: deferred reference escapes.
  read(SQL`SELECT ${query(dynamicSql)} AS value`); // BIND_ESCAPE: nested SQL call stays unknown.
}
export function shadow(SQL: any, options: QueryOptions) {
  read(SQL`SELECT ${options} AS value`); // BIND_UNKNOWN: tag binding is shadowed.
}
export function returnedBind(options: QueryOptions) {
  return SQL`SELECT ${options.id} AS id`; // BIND_CLEAR: no executor call is needed to classify a bind.
}
export function unsupportedReceiverBind(options: QueryOptions | undefined) {
  read(SQL`SELECT ${options} AS value`); // BIND_CLEAR: the receiver is only bound data here.
}
