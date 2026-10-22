import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
// An empty pattern introduces no row-derived name; the computed source is finite.
const [] = result.rows;
const { ["schema"]: schema } = { schema: "public" };
query(`SELECT id FROM ${schema}.users`);
