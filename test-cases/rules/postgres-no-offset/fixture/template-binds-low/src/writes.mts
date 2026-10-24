import { read } from '@example/db';
import SQL, { SQL as updated, default as destructured } from 'sql-template-strings';
import * as library from 'sql-template-strings';
export function writes(replacement: any, value: unknown) {
  SQL = replacement;
  updated++;
  ({ tag: destructured } = replacement);
  library.SQL = replacement;
  read(SQL`SELECT ${value}`); // unknown
  read(updated`SELECT ${value}`); // unknown
  read(destructured`SELECT ${value}`); // unknown
  read(library.SQL`SELECT ${value}`); // unknown
}
