import { query, sql } from "@example/db";
const suffix = sql` WHERE id = ${id}`;
const statement = sql`SELECT id FROM accounts`;
// Equivalent arms must keep both append occurrences after the choice folds.
if (flag) statement.append(suffix);
else statement.append(suffix);
query(statement);
query(sql`${statement}`);
