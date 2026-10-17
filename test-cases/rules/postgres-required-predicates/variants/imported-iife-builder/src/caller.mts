import sql from "sql-template-strings";
import { read } from "@example/db";
import { ownerVisibleSql } from "./visibility-sql.mts";

// A `const` bound to a directly invoked function that builds and returns one
// local builder takes that builder's final state.
export async function staticAppends(id: string) {
  const built = (() => {
    const query = sql`SELECT p.account_id FROM view_active_plans p WHERE p.account_id = ${id}`;
    query.append(sql` AND p.tier = 'a'`);
    return query;
  })();
  return read(built);
}

export async function conditionalAppend(id: string, flag: boolean) {
  const built = (function () {
    const query = sql`SELECT q.account_id FROM view_active_plans q WHERE q.account_id = ${id}`;
    if (!flag) {
      query.append(sql` AND q.tier = 'a'`);
    }
    return query;
  })();
  return read(built);
}

export async function withImportedHelper(id: string) {
  const built = (() => {
    const query = sql`SELECT t.account_id FROM tiers t WHERE t.id = ${id}`;
    query.append(ownerVisibleSql("t.parent_id", "t.account_id"));
    return query;
  })();
  return read(built);
}
