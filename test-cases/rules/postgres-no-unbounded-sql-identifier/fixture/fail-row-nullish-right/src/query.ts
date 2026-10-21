import { query } from "@example/db";
declare const maybe: unknown[] | undefined;
const rows = maybe ?? (await query(`SELECT schema FROM tenants`)).rows;
for (const row of rows) {
  query(`SELECT id FROM ${row.schema}.users`);
}
