import { query } from "@example/db";

export async function dynamicKeys(key: string, first: number, second: number) {
  await query(`SELECT pg_advisory_lock(hashtextextended($1, 0))`, [key]);
  await query(`SELECT pg_try_advisory_xact_lock($1, $2)`, [first, second]);
  await query(`SELECT pg_advisory_lock($1)`, [key]);
  await query(`SELECT pg_catalog.pg_advisory_lock_shared($1)`, [key]);
  await query(`SELECT pg_advisory_xact_lock($1)`, [key]);
  await query(`SELECT pg_advisory_xact_lock_shared($1)`, [key]);
  await query(`SELECT pg_try_advisory_lock($1)`, [key]);
  await query(`SELECT pg_try_advisory_lock_shared($1)`, [key]);
  await query(`SELECT pg_try_advisory_xact_lock_shared($1)`, [key]);
}

export async function unrelatedFunction(value: number) {
  await query(`SELECT pg_sleep($1)`, [value]);
}
