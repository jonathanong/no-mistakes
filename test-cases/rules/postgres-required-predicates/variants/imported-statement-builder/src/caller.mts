import sql from "sql-template-strings";
import { read } from "@example/db";
import { withTierFilter } from "./tier-filter.mts";

// The helper appends to the builder it receives and returns it. Both the
// caller's SELECT and the helper's appended subquery read the guarded view.
export async function planIds(tier: string) {
  return read(withTierFilter(sql`SELECT p.id FROM view_active_plans p WHERE true`, tier));
}

// The builder can also be a bound local rather than a tagged template.
export async function boundPlanIds(tier: string) {
  const query = sql`SELECT p.id FROM view_active_plans p WHERE p.tier IS NOT NULL`;
  return read(withTierFilter(query, tier));
}
