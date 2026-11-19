import sql from 'sql-template-strings';
import { read } from './db.mjs';
import { fixedJoin, selectedFields } from './projection.mjs';
export function conditionalFragment(flag: boolean, options: { query?: typeof read } | undefined) {
  const statement = sql`SELECT w.id FROM painted_widgets w`;
  if (flag) statement.append(fixedJoin);
  return read(statement, options);
}
export function fluentFragment(flag: boolean, options: { query?: typeof read } | undefined) {
  const statement = sql`SELECT `.append(selectedFields).append(sql` FROM painted_widgets w`).append(fixedJoin);
  if (flag) statement.append(sql` ORDER BY w.id`);
  return read(statement, options);
}
