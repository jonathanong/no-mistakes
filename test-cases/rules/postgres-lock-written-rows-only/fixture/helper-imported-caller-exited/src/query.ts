import { beginTransaction } from "@example/db";
import { writeUser } from "./helper";
export async function updateUser(userId: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [userId]);
  return;
  await writeUser(tx.query, userId);
}
