import sql from "sql-template-strings";
import { read } from "@example/db";
import { ownerVisibleSql } from "@services/links";
import { buildEligibilityFilter } from "@modules/feed-query-builders";
import { appendOrdering } from "./local-helpers.mts";

// An IIFE-local builder, a conditional tagged append, an imported
// literal-identifier helper through a `paths`-alias barrel, a second helper
// through another alias barrel, and a relative helper that appends to the
// builder it is given. Both views reads must be reported: the CTE below and
// the EXISTS in the services helper.
export async function viaIife(id: string, flag: boolean) {
  const built = (() => {
    const query = sql`WITH active_links AS (SELECT l.account_id FROM links l WHERE l.id = ${id}), `;
    query.append(sql`tiers AS (SELECT p.account_id FROM view_active_plans p WHERE p.account_id IN (SELECT account_id FROM active_links)) `);
    query.append(sql`SELECT t.account_id FROM tiers t WHERE true`);
    if (!flag) {
      query.append(sql` AND t.account_id IS NOT NULL`);
    }
    query.append(ownerVisibleSql("t.parent_id", "t.account_id"));
    query.append(buildEligibilityFilter("t", "t"));
    appendOrdering(query);
    return query;
  })();
  return read(built);
}

// The same builder without the IIFE.
export async function direct(id: string, flag: boolean) {
  const query = sql`WITH active_links AS (SELECT l.account_id FROM links l WHERE l.id = ${id}), `;
  query.append(sql`tiers AS (SELECT p.account_id FROM view_active_plans p WHERE p.account_id IN (SELECT account_id FROM active_links)) `);
  query.append(sql`SELECT t.account_id FROM tiers t WHERE true`);
  if (!flag) {
    query.append(sql` AND t.account_id IS NOT NULL`);
  }
  query.append(ownerVisibleSql("t.parent_id", "t.account_id"));
  query.append(buildEligibilityFilter("t", "t"));
  appendOrdering(query);
  return read(query);
}
