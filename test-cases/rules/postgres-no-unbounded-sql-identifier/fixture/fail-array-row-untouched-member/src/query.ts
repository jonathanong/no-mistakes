import { query } from "@example/db";
const result = await query(`SELECT schema, tenant FROM tenants`);
const [first] = result.rows;
first.schema = "public";
query(`SELECT id FROM ${first.tenant}.users`);
// Overwriting schema does not replace the row’s tenant field.
