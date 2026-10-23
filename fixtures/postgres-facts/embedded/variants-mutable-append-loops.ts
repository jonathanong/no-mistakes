import sql from "sql-template-strings";
import { query } from "@example/db";
// Mutable declarations still carry proven builders; one observed loop iteration
// must not erase the feasible zero-iteration SQL or claim exhaustive recovery.
let values = sql`SELECT 1`;
for (const value of list) values.append(" LIMIT 1");
query(values);
let classic = sql`SELECT 2`;
for (let i = 0; i < count; i++) classic.append(sql` WHERE id = ${i}`);
query(classic);
var pretest = sql`SELECT 3`;
while (condition) pretest.append(" LIMIT 3");
query(pretest);
let unchanged = sql`SELECT 4`;
while (condition) {}
query(unchanged);
