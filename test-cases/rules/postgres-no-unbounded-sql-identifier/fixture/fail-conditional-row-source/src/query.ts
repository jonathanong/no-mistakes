import { query } from "@example/db";
declare const flag: boolean;
const rows = flag ? (await query(`SELECT schema FROM tenants`)).rows : [];
for (const row of rows) {
  query(`SELECT id FROM ${row.schema}.users`);
}
