import { beginTransaction } from "@example/db";
export async function insertChild(parentId: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM parents WHERE id = $1 FOR UPDATE`, [parentId]);
  await tx.query(`INSERT INTO children (parent_id) VALUES ($1)`, [parentId]);
}
