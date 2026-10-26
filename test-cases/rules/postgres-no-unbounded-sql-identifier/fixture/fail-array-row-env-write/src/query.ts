import { query } from "@example/db";
const result = await query(`SELECT schema, tenant FROM tenants`);
const [first] = result.rows;
first.schema = process.env.SCHEMA;
query(`SELECT id FROM ${first.schema}.users`);
// Replacing a row field with another external source still needs validation.
