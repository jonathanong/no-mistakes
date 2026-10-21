import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
for (var row of result.rows) {}
row = { schema: "app" };
query(`SELECT id FROM ${row.schema}.users`);
