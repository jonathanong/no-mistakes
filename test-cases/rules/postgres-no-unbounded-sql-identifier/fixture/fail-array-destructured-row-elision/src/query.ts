import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
const [, { schema = "public" }] = result.rows;
query(`SELECT id FROM ${schema}.users`);
