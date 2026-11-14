import { query } from '@example/db';
import { assertIdentifier as safe } from '@example/identifiers';
import { assertIdentifier as other } from '@example/other';
const CONFIG = { user: { table: 'users' }, post: { table: 'posts' } } as const;
function mixed(kind: string) { switch (kind) { case 'user': return 'users'; default: return runtime(); } }
function incomplete(kind: string) { switch (kind) { case 'user': return 'users'; } }
export function run(kind: 'user' | 'post', table: string) {
  query(`LOCK TABLE ${mixed(kind)}`);
  query(`LOCK TABLE ${incomplete(kind)}`);
  query(`LOCK TABLE ${other(table, ['users', 'posts'])}`);
  query(`LOCK TABLE ${safe(table, ['users', runtime()])}`);
  query(`LOCK TABLE ${safe(table)}`);
  kind = runtime();
  query(`LOCK TABLE ${kind}`);
  const config = CONFIG[kind];
  config.table = runtime();
  query(`LOCK TABLE ${config.table}`);
}

export function optional(kind?: 'users' | 'posts') {
  query(`LOCK TABLE ${kind}`);
}
export function defaults(kind: 'users' | 'posts' = runtime()) {
  query(`LOCK TABLE ${kind}`);
}
export function shadow(safe: any, table: string) {
  query(`LOCK TABLE ${safe(table, ['users', 'posts'])}`);
}

export function mutated(kind: 'user' | 'post') {
  const config = CONFIG[kind];
  query(`SELECT 1 FROM ${config.table}`);
  config.table = runtime();
  query(`SELECT 1 FROM ${config.table}`);
}
export function deep(kind: 'a' | 'b') {
  const config = { a: { table: 'users', unused: { a: { a: { a: { a: { a: { a: { a: 'deep' } } } } } } } }, b: { table: 'posts' } } as const;
  query(`LOCK TABLE ${config[kind].table}`);
}
export function spreadBefore(parts: unknown[]) {
  query(`LOCK TABLE ${safe(...parts, ['users', 'posts'])}`);
}
