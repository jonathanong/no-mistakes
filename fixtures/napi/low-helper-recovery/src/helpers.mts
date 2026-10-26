import SQL from 'sql-template-strings';
import { read, type QueryExecutor, type QueryOptions } from '@example/db';
export function execute(query: QueryExecutor, id: string) {
  return query(SQL`SELECT ${id} AS id`);
}
export function forward(query: QueryExecutor, id: string) {
  return execute(query, id);
}
export function executeOptions(options: QueryOptions, id: string) {
  return read(SQL`SELECT ${id} AS id`, options);
}
export function fragment({ enabled = false, value }: { enabled?: boolean; value?: string } = {}) {
  const statement = SQL` WHERE 1 = 1`;
  if (enabled) statement.append(SQL` AND value = ${value}`);
  return statement;
}
export function offsetFragment({ offset = false }: { offset?: boolean } = {}) {
  return offset ? SQL` OFFSET 1` : SQL``; // OFFSET_ORIGIN
}
export function qualifier(alias: string) {
  return `${alias}.deleted_at IS NULL`;
}
export function fixedFragment() {
  return SQL` AND active = true`;
}
declare const opaque: (value: unknown) => unknown;
export function escaped(query: QueryExecutor, id: string) {
  opaque(query);
  return query(SQL`SELECT ${id} AS id`);
}
export function dynamicFragment(value: string) {
  return SQL` WHERE `.append(value);
}
export function escapedBuilder() {
  const statement = SQL` WHERE 1 = 1`;
  opaque(statement);
  return statement;
}
export function recursive(query: QueryExecutor): unknown {
  return recursive(query);
}
export function nestedBind(value: unknown) {
  return SQL`SELECT ${SQL`OFFSET 99`} AS data, ${value} AS value`;
}
export function invalidBoundRelation(table: string) {
  return SQL`SELECT * FROM ${table}`;
}
export function mixedFragment(enabled: boolean) {
  return enabled ? SQL` OFFSET 3` : opaque('fragment');
}
export async function asyncExecute(query: QueryExecutor, sql: string) {
  return await query(sql);
}
export async function asyncForward(query: QueryExecutor, sql: string) {
  return await asyncExecute(query, sql);
}
export async function asyncFragment() {
  return SQL` AND active = true`;
}
export async function asyncCarrier(query: QueryExecutor) {
  return query;
}
export async function unknownAwait(query: QueryExecutor, unknown: unknown) {
  await unknown;
  return query(SQL`SELECT 1`);
}
export function builderCarrier(query: QueryExecutor) {
  return SQL`SELECT ${query}`;
}
export function nestedBuilderCarrier(query: QueryExecutor) {
  return SQL`SELECT ${SQL`SELECT ${query}`}`;
}
export function consumeBuilderBind(query: QueryExecutor) {
  return read(SQL`SELECT ${query}`);
}
