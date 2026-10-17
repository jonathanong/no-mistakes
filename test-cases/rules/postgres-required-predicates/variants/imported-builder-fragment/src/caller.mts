import sql from "sql-template-strings";
import { beginTransaction } from "@example/db";
import { buildEligibilityFilter } from "./eligibility.mts";

// A straight-line builder whose CTE reads the guarded view, followed by an
// append from an imported fragment helper that takes (ignored) literal arguments.
export async function eligible(rootId: string) {
  const tx = beginTransaction();
  const query = sql`WITH roots AS (SELECT r.id FROM roots r WHERE r.id = ${rootId}), `;
  query.append(sql`tiers AS (SELECT p.id FROM view_active_plans p WHERE p.tier = 'a') `);
  query.append(sql`SELECT t.id FROM tiers t WHERE true`);
  query.append(buildEligibilityFilter("p", "root"));
  return tx.query(query);
}
