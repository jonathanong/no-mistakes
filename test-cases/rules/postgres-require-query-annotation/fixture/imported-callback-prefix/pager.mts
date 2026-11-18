import bindSql from 'sql-template-strings';
import { sql, type SqlStatement } from './sql.mjs';

type Run<T> = (statement: SqlStatement) => Promise<T>;

export async function page<T>(statement: SqlStatement, after: string | undefined, run: Run<T>) {
  const tail = after ? sql` AND id > ${after}` : sql``;
  return run(sql`${statement}${tail} ORDER BY id LIMIT 50`);
}

export async function prepend<T>(statement: SqlStatement, run: Run<T>) {
  return run(sql`SELECT 1; ${statement}`);
}

export function bound<T>(statement: unknown, run: Run<T>) {
  return run(bindSql`${statement} ORDER BY id LIMIT 50`);
}

declare function mutate(statement: SqlStatement): unknown;
export function mutated<T>(statement: SqlStatement, run: Run<T>) {
  return run(sql`${statement}${mutate(statement)} ORDER BY id LIMIT 50`);
}

export function shadowed<T>(statement: SqlStatement, sql: typeof import('./sql.mjs').sql, run: Run<T>) {
  return run(sql`${statement} ORDER BY id LIMIT 50`);
}
