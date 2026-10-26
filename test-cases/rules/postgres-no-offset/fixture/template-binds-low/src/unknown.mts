import { read } from '@example/db';
import SQL from 'sql-template-strings';
import { sql as custom } from '@example/custom-sql';
export function dynamic(value: string) {
  read(value); // unknown
  read(SQL`SELECT 1\nOFFSET 2`); // unknown: escaped quasi changes physical SQL lines.
  read(SQL`SELECT 1\
OFFSET 2`); // unknown: source continuation changes physical SQL lines.
  read(`SELECT ${value}`); // unknown
  read(custom`SELECT ${value}`); // unknown
  read(SQL`SELECT 1`.append(value)); // unknown
  const mutated = SQL`SELECT 1`;
  mutated.append(value);
  read(mutated); // unknown
  const aliased = SQL`SELECT 1`;
  const escaped = aliased;
  read(aliased); // unknown
  read(escaped); // unknown
}
export function shadow(SQL: typeof custom, value: unknown) {
  read(SQL`SELECT ${value}`); // unknown
}
