import { beginTransaction, writePool, type TransactionQuery } from "@example/db";

export async function opened(run: TransactionQuery) {
  const tx = await beginTransaction();
  await tx(`SELECT id FROM posts ORDER BY id OFFSET 10`);
  await run(`SELECT id FROM posts ORDER BY id OFFSET 11`);
  const client = await writePool.connect();
  return client.query(`SELECT id FROM posts ORDER BY id OFFSET 12`);
}
