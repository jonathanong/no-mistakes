import { query, sql } from "@example/db";
let nested = sql`SELECT 1`;
function mutate() { nested.append(" LIMIT 1"); }
query(nested);
let repeated = "SELECT 2";
while (flag) repeated += " LIMIT 1";
query(repeated);
let maybe;
if (flag) maybe = "SELECT 3";
query(maybe); // The absent path leaves the query uninitialized.
query(flag ? "SELECT 4" : sql.raw(variable));
query(sql`SELECT 5 ${sql.join([sql`WHERE active`, unknownFragment])}`);
