import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
const rows = result.rows ?? [];
for (const row of rows) {
  query(`SELECT id FROM ${row.schema}.users`);
}
