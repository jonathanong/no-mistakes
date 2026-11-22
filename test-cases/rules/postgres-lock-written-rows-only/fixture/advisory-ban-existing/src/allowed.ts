import { query } from "@example/db";

export async function intentionalAdvisoryLock(key: string) {
  await query(`SELECT pg_advisory_xact_lock(hashtextextended($1, 0))`, [key]);
}
