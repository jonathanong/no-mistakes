import { query } from '@example/db';
import sql from 'sql-template-strings';
import { assertIdentifier as safe } from '@example/identifiers';
const CONFIG = { user: { table: 'users', column: 'user_id' }, post: { table: 'posts', column: 'post_id' } } as const;
function column(kind: 'user' | 'post') {
  switch (kind) { case 'user': return 'user_id'; case 'post': return 'post_id'; }
}
function defaultColumn(kind: string) {
  switch (kind) { case 'user': return 'user_id'; default: return 'post_id'; }
}
function owned(table: string) {
  const allowed = new Set(['users', 'posts']);
  if (!allowed.has(table)) throw new Error('invalid identifier');
  return table;
}
export function run(kind: 'user' | 'post', table: string) {
  const config = CONFIG[kind];
  query(`DELETE FROM ${config.table} WHERE ${config.column} = $1`);
  query(`SELECT * FROM ${kind}`);
  const statement = sql`SELECT * FROM `;
  statement.append(safe(table, new Set(['users', 'posts'])));
  query(statement);
  const columns = sql`SELECT `;
  columns.append(column(kind));
  columns.append(' FROM users');
  query(columns);
  query(`LOCK TABLE ${safe(table, ['users', 'posts'])}`);
  query(`LOCK TABLE ${safe(table)}`);
  query(`LOCK TABLE ${owned(table)}`);
  const allowed = ['users', 'posts'] as const;
  query(`LOCK TABLE ${safe(table, allowed)}`);
  const set = new Set(['users', 'posts']);
  query(`LOCK TABLE ${safe(table, set)}`);
  const alias = set;
  query(`LOCK TABLE ${safe(table, alias)}`);
  query(`LOCK TABLE ${safe(table, new Set(allowed))}`);
  query(`SELECT ${defaultColumn(table)} FROM users`);
  const selected = safe(table, allowed);
  query(`LOCK TABLE ${selected}`);
  query(`SELECT ${column(kind)} FROM users`);
}
