import SQL from 'sql-template-strings';
import { read } from '@example/db';
declare const opaque: () => string;
export function member(flag: boolean) {
  return (flag ? { text: 'SELECT 1 OFFSET 1' } : null).text;
}
export function statement(flag: boolean) {
  if (flag) return SQL`SELECT 1 OFFSET 2`;
  return opaque();
}
export function append(flag: boolean) {
  return SQL`SELECT 1`.append(flag ? ' OFFSET 3' : null);
}
export function template(flag: boolean) {
  return `SELECT 1${flag ? ' OFFSET 4' : null}`;
}
export function partials(flag: boolean) {
  read(member(flag)); // PARTIAL_MEMBER
  read(statement(flag)); // PARTIAL_STATEMENT
  read(append(flag)); // PARTIAL_APPEND
  read(template(flag)); // PARTIAL_TEMPLATE
}
