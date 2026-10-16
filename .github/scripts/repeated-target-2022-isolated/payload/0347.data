import { query, sql } from "@example/db";
const gate = runtimeFlag;
query(sql`SELECT ${gate ? sql`1` : sql`2`}${gate ? sql`0` : sql`9`}`);
let text = "SELECT ";
if (gate) text += "1"; else text += "2";
if (gate) text += "0"; else text += "9";
query(text); // The unchanged const condition cannot choose different arms later.
query(sql`SELECT ${gate ? sql`1` : sql`2`}${gate && sql`0`}`);
query(sql`SELECT ${gate ? sql`1` : sql`2`}${!gate ? sql`0` : sql`9`}`);
let inverse = "SELECT ";
if (gate) inverse += "1"; else inverse += "2";
if (!gate) inverse += "0"; else inverse += "9";
query(inverse);
