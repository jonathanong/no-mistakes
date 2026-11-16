import SQL from 'sql-template-strings';
import { query } from '@example/db';

const HARD_MAX = 100;
const LABEL = 'active';

export function run(id: string, relation: string) {
  query(SQL`SELECT ${id}, ${LABEL} FROM generate_series(1, ${HARD_MAX}) LIMIT ${HARD_MAX}`);
  query(SQL`SELECT ${id}`.append(SQL` WHERE ${LABEL} = ${id}`));
  query(SQL`SELECT * FROM entries WHERE id = ${id}`.append(unknownFragment(relation)));
}
