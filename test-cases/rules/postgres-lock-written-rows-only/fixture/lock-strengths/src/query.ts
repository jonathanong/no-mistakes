import { beginTransaction } from "@example/db";
export async function allStrengths() {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM parents FOR UPDATE`);
  await tx.query(`SELECT id FROM parents FOR NO KEY UPDATE`);
  await tx.query(`SELECT id FROM parents FOR SHARE`);
  await tx.query(`SELECT id FROM parents FOR KEY SHARE`);
}
