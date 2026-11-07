import { query, read, write, run, openTransaction, type TxExecutor } from '@example/db';
import * as db from '@example/db';
await query('SELECT 1');
await read('SELECT 2');
await write('SELECT 3');
await run('SELECT 4');
await db.query('SELECT 5');
const transaction = openTransaction();
await transaction.query('SELECT 6');
export async function execute(transaction: TxExecutor) {
  await transaction.query('SELECT 7');
}
