import { query } from "@example/db";
const { rows = [] } = await query(`SELECT schema FROM tenants`);
for (const row of rows) {
  query(`SELECT id FROM ${row.schema}.users`);
}
