import { query } from "@example/db";
const result = await query(`SELECT schema, tenant FROM tenants`);
const [first] = result.rows;
first.schema = "public";
query(`SELECT id FROM ${first.schema}.users`);
// Only the adjacent direct scalar write proves this exact field no longer comes from the row.
