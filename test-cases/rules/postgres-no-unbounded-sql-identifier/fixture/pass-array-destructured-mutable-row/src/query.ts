import { query } from "@example/db";
const result = await query(`SELECT schema FROM tenants`);
// Mutable array patterns deliberately have no query-row provenance proof.
let [first] = result.rows;
first = { schema: "public" };
query(`SELECT id FROM ${first.schema}.users`);
