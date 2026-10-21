import { query } from "@example/db";
// A configured executor may return rows directly; a mutable binding can replace one.
let [first] = await query(`SELECT schema FROM tenants`);
first = { schema: "public" };
query(`SELECT id FROM ${first.schema}.users`);
