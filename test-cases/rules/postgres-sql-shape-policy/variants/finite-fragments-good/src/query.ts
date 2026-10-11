import { query, sql } from "@example/db";
const statement = sql`SELECT id FROM accounts`;
statement.append(flag ? sql` WHERE id = 1` : sql` WHERE id = 2`);
query(statement);
