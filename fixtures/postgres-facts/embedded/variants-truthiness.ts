import { query, sql } from "@example/db";
// A skipped operand is allowed to be opaque because it cannot become the query.
query("SELECT 1" || unknownSql);
query("" || "SELECT 2");
query("SELECT 3" ?? unknownSql);
query(null ?? "SELECT 4");
query(sql`SELECT 5 ${true && sql`LIMIT 1`}`);
query(sql`SELECT 6 ${false && sql`LIMIT 1`}`);
query(true ? "SELECT 7" : unknownSql);
query(false ? unknownSql : "SELECT 8");
let prefix;
let suffix;
if (flag) { prefix = ""; suffix = "9"; }
else { prefix = "SELECT 10"; suffix = ";"; }
query((prefix || "SELECT ") + suffix); // Never SELECT ; or SELECT 109.
let fixed = "SELECT 11";
if (false) fixed += " OFFSET 1";
query(fixed);
