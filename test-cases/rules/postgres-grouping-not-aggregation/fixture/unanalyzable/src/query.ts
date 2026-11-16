import { query, sql } from "db";

interface OtherClient {
  query(statement: string): Promise<unknown>;
}

export async function groupingPolicy(other: OtherClient) {
  await query(sql`SELECT DISTINCT id FROM policy_items`);
  await other.query("SELECT DISTINCT id FROM policy_items");
}
