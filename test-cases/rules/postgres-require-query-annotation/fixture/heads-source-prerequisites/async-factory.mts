import sql from 'sql-template-strings';
import { add } from './broken.mjs';
export async function factory() {
  const statement = sql`/* asyncEffect */ SELECT 1`;
  add(statement);
  return statement;
}
