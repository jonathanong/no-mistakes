import { beginTransaction } from "@example/db";
import { lockUser as acquireUser } from "./helper";
export async function updateUser(userId: string) {
  const tx = await beginTransaction();
  await acquireUser(tx.query, userId);
  await tx.query(`UPDATE users SET value = 1 WHERE id = $1`, [userId]);
}
