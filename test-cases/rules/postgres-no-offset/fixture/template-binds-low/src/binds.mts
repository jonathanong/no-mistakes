import { read } from '@example/db';
import SQL, { SQL as renamed, default as alternate } from 'sql-template-strings';
import * as library from 'sql-template-strings';
export function binds(value: unknown) {
  read(SQL`SELECT ${value} AS id`);
  read(renamed`SELECT ${value} AS id`);
  read(alternate`SELECT ${value} AS id`);
  read(library.SQL`SELECT ${value} AS id`);
  read(library['default']`SELECT ${value} AS id`);
  // Even SQL-looking values and nested statements are binds in this library.
  read(SQL`SELECT ${'OFFSET 50'} AS text`);
  read(SQL`SELECT ${SQL`SELECT 1`} AS statement`);
  read(SQL`SELECT ${process.env.TABLE} AS value`);
  const statement = SQL`SELECT ${value} AS id`;
  read(statement);
  read(SQL`SELECT 1`);
}
export function multiline(value: unknown) {
  read(SQL`SELECT ${
    value
  } AS id
OFFSET 2`); // offset: preserve lines hidden inside the interpolation.
}
export function comment(value: unknown) {
  // Interpolation source newlines must not terminate the actual SQL comment.
  read(SQL`SELECT 1 -- ${
    value
  } OFFSET 99`);
}
export function sameSqlLine(value: unknown) {
  read(SQL`SELECT ${
    value
  } AS id OFFSET 3`); // offset: SQL column distinguishes source lines.
}
