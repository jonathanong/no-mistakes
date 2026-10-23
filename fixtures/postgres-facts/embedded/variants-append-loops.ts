import sql from "sql-template-strings";
import { query } from "@example/db";
// Loop effects are deliberately approximated by zero and one iteration.
const classic = sql`SELECT 1`;
for (let i = 0; i < count; i++) classic.append(" /* classic */");
query(classic);
const keys = sql`SELECT 2`;
for (const key in object) keys.append(" /* keys */");
query(keys);
const values = sql`SELECT 3`;
for (const value of list) values.append(sql` WHERE id = ${value}`);
query(values);
const pretest = sql`SELECT 4`;
while (condition) pretest.append(" /* while */");
query(pretest);
const posttest = sql`SELECT 5`;
do posttest.append(" /* do */"); while (condition);
query(posttest);
const unchanged = sql`SELECT 6`;
while (condition) {}
query(unchanged);
