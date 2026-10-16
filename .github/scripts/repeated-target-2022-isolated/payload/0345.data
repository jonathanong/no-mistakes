import { query, sql } from "@example/db";
const yes = true;
query(yes ? "SELECT 1" : unknownSql);
const no = false;
query(no ? unknownSql : "SELECT 2");
const nothing = null;
query(nothing ? unknownSql : "SELECT 3");
const nonempty = "ready";
query(nonempty ? "SELECT 4" : unknownSql);
const empty = "";
query(empty ? unknownSql : "SELECT 5");
const gate = runtimeFlag;
query(gate ? (gate ? "SELECT 6" : unknownSql) : "SELECT 7");
const derived = gate ? true : false;
query(sql`SELECT ${gate ? sql`1` : sql`2`}${derived ? sql`0` : sql`9`}`);
const inverse = gate ? false : true;
query(sql`SELECT ${gate ? sql`1` : sql`2`}${inverse ? sql`0` : sql`9`}`);
let condition = "ready";
let q = "SELECT ";
if (condition) { condition = ""; q += "8"; }
else q += "9";
query(q); // The test observes the value before its taken branch changes it.
