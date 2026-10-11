import { query, sql } from "@example/db";
query(flag ? "SELECT 1" : unknownSql);
query(sql`SELECT 1 ${sql.raw(variable)}`);
query(sql`SELECT 1 ${sql.join(variable)}`);
query(sql`SELECT 1 ${sql.join([...fragments])}`);
query(flag ? "SELECT 1" : `SELECT ${sqlText}`);
let q = sql`SELECT 1`;
for (const item of items) q.append(" LIMIT 1");
query(q);
query(externalHelper());
for (const item of items) query(flag ? "SELECT 2" : "SELECT 3");
const staticText = "WHERE active";
query(sql`SELECT 1 ${sql.raw(staticText)}`); // raw requires a literal at the call.
query(sql`SELECT 1 ${sql.join([, sql`WHERE active`])}`);
query(sql`SELECT 1 ${sql.join([sql`WHERE active`], separator)}`);
query(sql`SELECT 1 ${sql[method]("WHERE active")}`);
query(sql`SELECT 1 ${sql` ${sql` ${sql` ${sql` ${sql` ${sql` ${sql` ${sql` ${sql` ${sql`LIMIT 1`}`}`}`}`}`}`}`}`}`}`);
