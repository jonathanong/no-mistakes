import { query } from '@example/db';
import sql from 'sql-template-strings';
import { sql as raw } from '@example/raw-sql';
const CONFIG = { user: { table: 'users' }, post: { table: 'posts' } } as const;
export function run(flag: boolean, kind: 'user' | 'post') {
  const config = CONFIG[kind];
  query(flag ? sql`DROP TABLE users` : raw`SELECT 1 FROM ${config.table}`);
  escape(config);
}
