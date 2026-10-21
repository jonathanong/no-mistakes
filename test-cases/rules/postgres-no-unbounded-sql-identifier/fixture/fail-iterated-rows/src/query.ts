import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
for (const row of result.rows) {
  query(`SELECT id FROM ${row.schema}.users`);
}
