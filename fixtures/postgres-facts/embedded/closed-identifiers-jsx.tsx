import { query } from '@example/db';
import { sql as raw } from '@example/raw-sql';
export function jsxChild(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  const rendered = <Comp child={config.child} />;
  render(rendered);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
export function jsxScalar(kind: 'user' | 'post') {
  const CONFIG = { user: { child: { table: 'users' } }, post: { child: { table: 'posts' } } } as const;
  const config = CONFIG[kind];
  const rendered = <Comp child={config.child.table} />;
  render(rendered);
  query(raw`SELECT 1 FROM ${config.child.table}`);
}
