import { sql as raw } from '@example/raw-sql';
import sql from 'sql-template-strings';

export function rawStatement(table: string) {
  const statement = raw`SELECT id FROM ${table} WHERE id = $1`;
  return statement;
}
export function rawConstant() {
  return raw`SELECT 1`;
}
export function rawChoice(kind: string) {
  const statement = kind === 'drop' ? 'DROP TABLE users' : 'SELECT id FROM users';
  return raw`${statement}`;
}
export function rawComposed(table: string) {
  return rawStatement(table);
}
export function boundValue(value: string) {
  return sql`SELECT id FROM users WHERE name = ${value}`;
}
