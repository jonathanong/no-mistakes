import { query, sql, type OtherClient } from "db";

export async function groupingPolicy(other: OtherClient) {
  await query(sql`SELECT DISTINCT id FROM policy_items`);
  await other.query("SELECT DISTINCT id FROM policy_items");
}
