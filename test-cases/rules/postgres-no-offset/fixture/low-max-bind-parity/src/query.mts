import SQL from 'sql-template-strings';
import { read } from '@example/db';

export function statements(value: string, id: string) {
  // A PostgreSQL bind cannot substitute a relation, even if max publishes an identifier marker.
  read(SQL`SELECT 1 FROM ${value}`); // UNKNOWN_RELATION
  read(SQL`SELECT ${value} AS value OFFSET 1`); // BOUND_OFFSET
  read(SQL`SELECT 1 FROM accounts WHERE id::text = ${id}`); // TRANSFORMED_UUID
  read('SELECT 1 OFFSET 2'); // ORDINARY_LITERAL
  read(SQL`SELECT ${value} AS value
    OFFSET 3`); // MULTILINE_OFFSET
  const statement = SQL`SELECT 1
    OFFSET 4`; // LOCAL_TAG_OFFSET
  read(statement);
}
