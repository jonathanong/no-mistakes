import { beginTransaction } from "@example/db";
import { sql } from "@example/sql";
export async function relationAllowMustNotMatchGeneratedMarker(target: string) {
  const tx = await beginTransaction();
  await tx.query(sql`LOCK TABLE ${target} IN EXCLUSIVE MODE`);
  await tx.query(sql`LOCK TABLE sql_placeholder_1 IN EXCLUSIVE MODE`);
}
