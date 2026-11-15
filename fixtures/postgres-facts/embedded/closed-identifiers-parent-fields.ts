import { query } from '@example/db';
function tableName() { return 'users'; }
export function computed(id: string) {
  const table = tableName();
  query(`SELECT id FROM ${table} WHERE id = $1`, [id]);
}
export function nested(id: string) {
  const CONFIG = { user: { table: 'users', column: 'id' }, post: { table: 'posts', column: 'id' } } as const;
  const config = CONFIG['user'];
  query(`SELECT id FROM ${config.table} WHERE ${config.column} = $1`, [id]);
}
export function poisoned(id: string) {
  const CONFIG = { user: { table: 'users', column: 'id' } } as const;
  const config = CONFIG['user'];
  escape(config);
  query(`SELECT id FROM ${config.table} WHERE ${config.column} = $1`, [id]);
}
