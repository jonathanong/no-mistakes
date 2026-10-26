import { query } from "@example/db";
const result = await query(`SELECT schema, nested, tenant FROM tenants`);
const [first] = result.rows;
declare const key: string;
first[key] = "public";
query(`SELECT id FROM ${first.schema}.users`);
first.schema = "public";
query(`SELECT id FROM ${first.tenant}.users`, []);
first.nested.schema = "public";
query(`SELECT id FROM ${first.tenant}.users`);
first.schema = "public";
query("SELECT id FROM users");
first.schema = "public";
query(`SELECT id FROM ${first.nested.tenant}.users`);
globalThis.schema = "public";
query(`SELECT id FROM ${globalThis.schema}.users`);
// Computed or unrelated writes do not make untouched row projections finite.
