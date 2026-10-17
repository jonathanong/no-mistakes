import { query, sql } from "@example/db";
query(sql`SELECT ${flag ? false : sql`1`}`); // A scalar Boolean arm is a bind, not absence.
query(sql`SELECT ${flag ? true : sql`2`}`);
query(sql`SELECT ${flag ? null : sql`3`}`);
query(sql`SELECT 4 ${null && sql`LIMIT 1`}`);
query(sql`SELECT 5 ${"" && sql`LIMIT 1`}`);
query(sql`SELECT 6 ${false && sql`LIMIT 1`}`);
query(sql`SELECT 7 ${(flag ? "" : sql`ignored`) && sql`LIMIT 1`}`);
query(flag ? false : "SELECT 8");
query(true);
query(false);
query((flag ? false : null) ?? "SELECT 9");
