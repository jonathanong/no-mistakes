import { beginTransaction, sql } from '@example/db';
export async function alias(userId: string, callback: () => Promise<void>) {
  await using transaction = await beginTransaction();
  const query = transaction;
  await query(sql`SELECT id FROM users WHERE id = ${userId} FOR UPDATE`);
  await callback();
  await transaction.commit();
}
