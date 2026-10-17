import sql from "sql-template-strings";
import { read } from "@example/db";
import { ownerVisibleSql } from "./visibility-sql.mts";

// The imported hole is filled in every alternative of the conditional builder.
export async function helperAfterConditional(id: string, flag: boolean) {
  const query = sql`SELECT t.account_id FROM tiers t WHERE t.id = ${id}`;
  if (!flag) {
    query.append(sql` AND t.account_id IS NOT NULL`);
  }
  query.append(ownerVisibleSql("t.parent_id", "t.account_id"));
  return read(query);
}

// One alternative reads the view in the caller itself.
export async function readInOneAlternative(id: string, flag: boolean) {
  const query = sql`SELECT t.account_id FROM tiers t WHERE t.id = ${id}`;
  if (flag) {
    query.append(sql` AND t.account_id IN (SELECT z.account_id FROM view_active_plans z)`);
  }
  query.append(ownerVisibleSql("t.parent_id", "t.account_id"));
  return read(query);
}

// Five independent conditionals make 32 alternatives, past the cap of 16.
export async function pastTheVersionCap(id: string, a: boolean, b: boolean, c: boolean, d: boolean, e: boolean) {
  const query = sql`SELECT t.account_id FROM tiers t WHERE t.id = ${id}`;
  if (a) query.append(sql` AND t.a IS NOT NULL`);
  if (b) query.append(sql` AND t.b IS NOT NULL`);
  if (c) query.append(sql` AND t.c IS NOT NULL`);
  if (d) query.append(sql` AND t.d IS NOT NULL`);
  if (e) query.append(sql` AND t.e IS NOT NULL`);
  query.append(ownerVisibleSql("t.parent_id", "t.account_id"));
  return read(query);
}
