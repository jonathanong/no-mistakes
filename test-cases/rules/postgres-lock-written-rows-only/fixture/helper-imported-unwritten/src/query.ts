import { beginTransaction } from "@example/db";
import { lockUser as acquireUser } from "./helper";
export async function updateMembership(userId: string) {
  const tx = await beginTransaction();
  await acquireUser(tx.query, userId);
  await tx.query(`UPDATE memberships SET value = 1 WHERE user_id = $1`, [userId]);
}
