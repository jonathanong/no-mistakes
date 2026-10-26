import { query } from "@example/db";
const result = await query(`SELECT schema, tenant FROM tenants`);
const [first] = result.rows;
declare const flag: boolean;
if (flag) first.schema = "public";
query(`SELECT id FROM ${first.schema}.users`);
// The original row field remains possible when the branch is not taken.
