import { query } from "@example/db";

export async function list() {
  const result = await query(`SELECT schema FROM tenants`);
  return query(`SELECT id FROM ${result.rows[0].schema}.users`);
}
