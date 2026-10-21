import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
const { schema } = result.rows[0];
query(`SELECT id FROM ${schema}.users`);
