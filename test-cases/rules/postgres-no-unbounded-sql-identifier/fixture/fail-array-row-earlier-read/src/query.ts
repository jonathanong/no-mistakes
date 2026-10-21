import { query } from "@example/db";
const result = await query(`SELECT schema, tenant FROM tenants`);
const [first] = result.rows;
query(`SELECT id FROM ${first.schema}.users`);
first.schema = "public";
query(`SELECT id FROM ${first.schema}.users`);
// A later scalar write must not erase the earlier captured row read.
