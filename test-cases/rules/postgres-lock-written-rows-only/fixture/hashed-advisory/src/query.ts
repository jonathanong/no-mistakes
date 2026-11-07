import { beginTransaction } from "@example/db";
const GLOBAL_KEY = "merge-admission";
const SEED = 0;
export async function hashedKeys(rowId: string, seed: number) {
  const tx = await beginTransaction();
  await tx.query(`SELECT pg_advisory_xact_lock(hashtext('global'))`);
  await tx.query(`SELECT pg_advisory_xact_lock(hashtextextended('global', 0))`);
  await tx.query(`SELECT pg_advisory_xact_lock(pg_catalog.hashtext($1))`, [GLOBAL_KEY]);
  await tx.query(`SELECT pg_advisory_xact_lock(hashtextextended($1, $2))`, [GLOBAL_KEY, SEED]);
  await tx.query(`SELECT pg_advisory_xact_lock(hashtextextended($1, 0))`, [rowId]);
  await tx.query(`SELECT pg_advisory_xact_lock(hashtextextended('global', $1))`, [seed]);
  await tx.query(`SELECT pg_advisory_xact_lock(other.hashtext('global'))`);
  await tx.query(`SELECT pg_advisory_xact_lock(hashtext(NULL))`);
  await tx.query(`SELECT pg_advisory_xact_lock(hashtext('global', 0))`);
  await tx.query(`SELECT pg_advisory_xact_lock(hashtextextended('global'))`);
  await tx.query(`SELECT pg_advisory_xact_lock(hashtext(value => 'global'))`);
  await tx.query(`SELECT pg_advisory_xact_lock(hashtext($1)::bigint)`, [rowId]);
}
