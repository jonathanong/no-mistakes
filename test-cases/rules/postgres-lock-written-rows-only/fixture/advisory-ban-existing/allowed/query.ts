import { query } from "@example/db";

export async function includedByGlob(key: string) {
  await query(`SELECT pg_try_advisory_lock_shared(hashtext($1))`, [key]);
}
