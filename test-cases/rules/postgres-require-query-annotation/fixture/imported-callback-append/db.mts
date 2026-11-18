import type { SqlStatement } from './sql.mjs';
export function read(statement: SqlStatement): Promise<void> { return Promise.resolve(); }
export function readRows(statement: SqlStatement, options?: { query?: string }): Promise<{ rows: unknown[] }> { return Promise.resolve({ rows: [] }); }
export function transaction<T>(run: (tx: (statement: SqlStatement) => Promise<{ rows: unknown[] }>) => Promise<T>): Promise<T> { return run(readRows); }
