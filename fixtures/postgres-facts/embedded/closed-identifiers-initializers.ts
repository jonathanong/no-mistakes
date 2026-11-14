import { query } from '@example/db';
import { sql as raw } from '@example/raw-sql';
export function classChild(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  class Holder { child = config.child; }
  escape(new Holder());
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function accessorChild(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  class Holder { accessor child = config.child; }
  escape(new Holder());
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function staticChild(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  class Holder { static child = config.child; }
  escape(Holder);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function defaultChild(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  function expose(child = config.child) { escape(child); }
  expose();
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function classScalar(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  class Holder { child = config.child.table; }
  escape(new Holder());
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function defaultScalar(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  function expose(child = config.child.table) { escape(child); }
  expose();
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
