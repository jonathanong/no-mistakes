import { beginTransaction } from "@example/db";
export async function uncalledHelper(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  async function neverCalled() {
    await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
  }
}
