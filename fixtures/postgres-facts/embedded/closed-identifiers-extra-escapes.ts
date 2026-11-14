import { query } from '@example/db';
import { sql as raw } from '@example/raw-sql';
export function storedObject(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  const wrapper = { child: config.child };
  escape(wrapper);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function storedArray(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  const wrapper = [config.child];
  escape(wrapper);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function* yieldedChild(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  yield config.child;
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function thrownChild(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  try { throw config.child; } catch (error) { escape(error); }
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function iteratedChild(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  for (const child of [config.child]) { escape(child); }
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
