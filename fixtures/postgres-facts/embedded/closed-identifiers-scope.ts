import { query } from '@example/db';
import { assertIdentifier as safe } from '@example/identifiers';
import { sql as raw } from '@example/raw-sql';
export function escaped(kind: 'user' | 'post') {
  const CONFIG = { user: { table: 'users' }, post: { table: 'posts' } } as const;
  const config = CONFIG[kind];
  query(raw`SELECT 1 FROM ${config.table}`);
  escape(config);
}
export function clean(kind: 'user' | 'post') {
  const CONFIG = { user: { table: 'users' }, post: { table: 'posts' } } as const;
  const config = CONFIG[kind];
  query(raw`SELECT 1 FROM ${config.table}`);
}
export function nested(kind: 'user' | 'post') {
  const CONFIG = { user: { table: 'users' }, post: { table: 'posts' } } as const;
  { const CONFIG = { user: { table: 'other' }, post: { table: 'another' } } as const; escape(CONFIG); }
  query(raw`SELECT 1 FROM ${CONFIG[kind].table}`);
}
export function changed(table: string) {
  const names = ['users', 'posts'];
  query(raw`LOCK TABLE ${safe(table, names)}`);
  names.push(runtime());
}
export function stable(table: string) {
  const names = ['users', 'posts'];
  query(raw`LOCK TABLE ${safe(table, names)}`);
}
