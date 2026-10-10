import { query, sql } from "@example/db";

query(sql`${sql`/* users/list */ SELECT 1`}`); // unknown
query(sql`${sql.raw(runtime)}`); // unknown
query(sql`/* users/list */ SELECT id FROM users ${sql`WHERE active`}`);
query(sql`SELECT id FROM users ${sql`WHERE active`}`); // missing
const inner = sql`/* users/list */ SELECT 1`;
query(sql`${inner}`); // unknown
function fragment() { return sql`/* users/list */ SELECT 1`; }
query(sql`${fragment()}`); // unknown

query(sql`${sql`/* users/list */ SELECT 1`} ${42}`); // first effect is a fragment
query(sql`${cond ? inner : sql`SELECT 1`}`); // possible builders
query(sql`${[inner, sql`SELECT 1`]}`); // aggregate builders stay opaque
query(sql`${(holder.sql).raw(runtime)}`); // nested member stays opaque
query(sql`${inner = sql`SELECT 1`}`); // evaluated assignment
query(sql`${holder[0] = sql`SELECT 1`}`); // evaluated property assignment
