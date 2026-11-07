import { query, openTransaction } from "@example/db";
export async function branch(id: string, choose: boolean) {
  const tx = await openTransaction();
  const sql = choose ? 'SELECT id FROM items FOR UPDATE' : 'SELECT id FROM parents FOR UPDATE';
  await tx.query(sql, [id]);
}
