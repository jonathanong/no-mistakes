import { beginTransaction } from "@example/db";
export async function opaqueWrite(id: string, sql: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(sql, [id]);
}
