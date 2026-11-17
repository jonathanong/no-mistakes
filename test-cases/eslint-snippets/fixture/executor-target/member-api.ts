import type { QueryExecutor } from '@example/db';

export function load(query: QueryExecutor) { return query('SELECT id FROM t'); }
export const arrow = (query: QueryExecutor) => query('SELECT id FROM t');
export function dynamic(query: QueryExecutor, statement: string) { return query(statement); }
export const mutable = { load(query: QueryExecutor) { return query('SELECT id FROM t'); } };
export function options({ query }: { query: QueryExecutor }) { return query('SELECT id FROM t'); }
