import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
const [first] = result.rows;
first.schema = "public";
const alias = first;
alias.schema = process.env.SCHEMA;
query(`SELECT id FROM ${first.schema}.users`);
// An alias can restore an external value after the finite write.
