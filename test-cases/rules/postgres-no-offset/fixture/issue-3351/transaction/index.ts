import { transaction } from 'db-lib';
const q = (input: string, values: unknown[]) => transaction(input, values);
const withExtra = (statement: string, binds: unknown[], ignored: unknown) => transaction(statement, binds);
q('UPDATE items SET a = 1 WHERE id = $1', [id]);
withExtra('UPDATE items SET a = 2 WHERE id = $1', [id], null);
