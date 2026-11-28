import { query, sql } from '@example.invalid/db';

export async function runManualMigration() {
  await query(sql`BEGIN`);
  await query(sql`SELECT 1`);
  await query(sql`COMMIT`);
}
