import { query } from '@example/db';
import { sql as raw } from '@example/raw-sql';
export function taggedChild(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  unknownTag`${config.child}`;
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function defaultBinding(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  const { child = config.child } = runtime();
  escape(child);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function assignedDefault(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  let child;
  ({ child = config.child } = runtime());
  escape(child);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function spreadProps(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { nested: { table: 'users' } } }, post: { child: { nested: { table: 'posts' } } } } as const;
  const config = CONFIG[kind];
  const rendered = <Comp {...config.child} />;
  render(rendered);
  query(raw`SELECT 1 FROM ${config.child.nested.table}`);
}
export function unknownKey(kind: 'user' | 'post', key: string) {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  escape(config[key]);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function taggedScalar(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  unknownTag`${config.child.table}`;
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function defaultScalar(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  const { child = config.child.table } = runtime();
  escape(child);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function spreadScalar(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  const rendered = <Comp {...{ label: config.child.table }} />;
  render(rendered);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
const EXPORTED = { user: { child: { table: 'users' } } } as const;
const exportedConfig = EXPORTED.user;
export default exportedConfig.child;
export function exportedChild() {
  query(raw`SELECT 1 FROM ${exportedConfig.child.table}`);
}
