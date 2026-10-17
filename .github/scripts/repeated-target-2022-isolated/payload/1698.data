import { query, sql } from "@example/db";
const gate = runtimeFlag;
query((gate ? null : "SELECT 1") ?? (gate ? "SELECT 2" : unknownSql));
query((gate ? "" : "SELECT 3") || (gate ? "SELECT 4" : unknownSql));
query(sql`SELECT 5 ${(gate ? sql`ignored` : null) && (gate ? sql`LIMIT 1` : sql.raw(unknownSql))}`);
