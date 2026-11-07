import { beginTransaction } from "@example/db";
export async function lockTables() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE items IN ACCESS SHARE MODE`);
  await tx.query(`LOCK TABLE parents IN EXCLUSIVE MODE`);
}
