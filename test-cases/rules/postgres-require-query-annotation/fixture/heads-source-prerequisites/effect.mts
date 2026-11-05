import sql from 'sql-template-strings';
import { read } from '@bench/db';
import { add } from './broken.mjs';
function run(tail) {
  const statement = sql`/* effect */ SELECT 1`;
  add(statement, tail);
  read(statement);
  read('SELECT unrelated');
}
