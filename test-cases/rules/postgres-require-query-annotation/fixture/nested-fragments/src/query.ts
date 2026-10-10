import { query, sql } from "@example/db";

query(sql`${sql`/* users/list */ SELECT 1`}`); // unknown
query(sql`${sql.raw(runtime)}`); // unknown
query(sql`/* users/list */ SELECT id FROM users ${sql`WHERE active`}`);
query(sql`SELECT id FROM users ${sql`WHERE active`}`); // missing
const inner = sql`/* users/list */ SELECT 1`;
query(sql`${inner}`); // unknown
function fragment() { return sql`/* users/list */ SELECT 1`; }
query(sql`${fragment()}`); // unknown
