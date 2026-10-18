import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";

const SAFE = /^[a-z_]+$/;

// A call whose only effect is to throw: the path that throws never executes.
export function assertSafeAlias(alias: string): void {
  if (!SAFE.test(alias)) throw new Error(`Unsafe alias: ${alias}`);
}

export function deleted(a: string, b: string): SQLStatement {
  return sql`(`
    .append(`${a}.deleted_at IS NULL`)
    .append(sql` AND `)
    .append(`${b}.deleted_at IS NULL`)
    .append(sql`)`);
}

// The guarded view is read here, three helpers below the executor.
export function access(alias: string, includeDrafts: boolean): SQLStatement {
  const statement = sql`(`.append(`${alias}.owner_id IS NULL`);
  statement.append(sql` OR EXISTS (SELECT 1 FROM view_active_plans v WHERE v.tier = ${alias}.tier`);
  if (!includeDrafts) {
    statement.append(sql` AND v.draft_at IS NULL`);
  }
  return statement.append(sql`))`);
}
