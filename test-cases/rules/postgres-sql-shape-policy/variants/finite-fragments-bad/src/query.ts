import { query, sql } from "@example/db";
const statement = sql`SELECT id FROM accounts`;
statement.append(flag ? sql`
  WHERE pg_sleep(1) IS NULL
` : sql` WHERE id = 2`);
query(statement);
