import { beginTransaction } from "@example/db";
export async function run() {
  const tx = await beginTransaction();
  await tx.query(`SELECT pg_advisory_xact_lock(hashtextextended('global', 0))`);
}
