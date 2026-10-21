import { query } from "@example/db";
const { rows } = await query(`SELECT schema FROM tenants`);
query(`SELECT id FROM ${rows[0].schema}.users`);
