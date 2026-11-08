import { sql } from 'db';
export function statementSql() { return sql`SELECT a.id FROM accounts a `; }
