import { query, sql } from "@example/db";
query(sql`SELECT 24 ${0 && sql`OFFSET 1`}`);
query(sql`SELECT 25 ${1 && sql`LIMIT 1`}`);
query(0 || "SELECT 26");
query(-runtimeFlag ? "SELECT 27" : "SELECT 28");
let unknownPrefix = unknownSql;
unknownPrefix += " LIMIT 1";
query(unknownPrefix);
let unknownBuilder = unknownSql;
unknownBuilder.append(" LIMIT 1");
query(unknownBuilder);
query(sql`SELECT 29`.append());
query(sql.raw(`SELECT ${text}`));
query(sql.join());
query(sql.join([sql`SELECT 30`, sql`LIMIT 1`], null));
query((flag ? sql`SELECT 31` : null) && unknownSql);
const gate = runtimeFlag;
let selected = gate ? "SELECT 32" : "SELECT 33";
if (gate) query(selected);
else query(selected);
// The nested fragment guard is false on the only path that evaluates it.
query(gate ? sql`SELECT 34 ${!gate && sql`OFFSET 1`}` : sql`SELECT 35`);
const derived = gate ? true : false;
query(derived ? (gate ? "SELECT 36" : unknownSql) : (!gate ? "SELECT 37" : unknownSql));
let switchSql = "SELECT ";
switch ("a") {
  case unknownCase: switchSql += "38"; break;
  default: switchSql += "39";
}
query(switchSql);
let unreachable = "SELECT 40";
if (gate) {
  if (!gate) {
    if (derived) unreachable += " OFFSET 1";
  }
}
query(unreachable);
