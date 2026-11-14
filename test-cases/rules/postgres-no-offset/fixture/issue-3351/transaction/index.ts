import { transaction, sql } from 'db-lib';
const q = (input: unknown, values: unknown[]) => transaction(input, values);
const withExtra = (statement: string, binds: unknown[], ignored: unknown) => transaction(statement, binds);
await q(sql`UPDATE items SET a = 1 WHERE id = $1`, [id]);
withExtra('UPDATE items SET a = 2 WHERE id = $1', [id], null);
