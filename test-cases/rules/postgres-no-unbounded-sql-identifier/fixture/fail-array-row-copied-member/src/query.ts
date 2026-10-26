import { query } from "@example/db";
const result = await query(`SELECT schema, tenant FROM tenants`);
const [first] = result.rows;
const { schema } = first;
first.schema = "public";
query(`SELECT id FROM ${schema}.users`);
// The scalar schema was captured before the row member changed.
