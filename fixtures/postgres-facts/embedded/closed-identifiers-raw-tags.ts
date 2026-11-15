import { query } from '@example/db';
import sql from 'sql-template-strings';
import { sql as raw } from '@example/raw-sql';
const CONFIG = { user: { table: 'users', column: 'user_id' }, post: { table: 'posts', column: 'post_id' } } as const;
export function run(kind: 'user' | 'post', unknown: string) {
  const config = CONFIG[kind];
  query(raw`DELETE FROM ${config.table} WHERE ${config.column} = $1`);
  query(sql`SELECT * FROM users WHERE name = ${kind}`);
  query(raw`LOCK TABLE ${unknown}`);
}
