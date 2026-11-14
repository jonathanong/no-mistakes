import { query } from '@example/db';
import sql from 'sql-template-strings';
import { sql as raw } from '@example/raw-sql';
export function mixedBefore(kind: 'user' | 'post', mutate: boolean) {
  const CONFIG = { user: { table: 'users' }, post: { table: 'posts' } } as const;
  escape(CONFIG);
  const statement = mutate ? sql`DROP TABLE users` : raw`SELECT 1 FROM ${CONFIG[kind].table}`;
  query(statement);
}
export function escapedChild(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  escape(config.child);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function assignedChild(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  Object.assign(config.child, runtime());
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function scalarRead(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  consume(config.child.table);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function trackedMemberAlias(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  const child = config.child;
  query(raw`SELECT 1 FROM ${child.table}`);
}
export function mutableMemberAlias(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  let child = config.child;
  child.table = runtime();
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function arrowReturn(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  const getter = () => config.child;
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function wrappedEscape(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  escape({ child: config.child });
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function constructorEscape(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  new Wrapper(config.child);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
function finiteStatement(kind: 'read' | 'drop') {
  switch (kind) {
    case 'drop': return 'DROP TABLE users';
    default: return 'SELECT 1';
  }
}
export function returnedLiteral(kind: 'read' | 'drop') {
  query(finiteStatement(kind));
}
