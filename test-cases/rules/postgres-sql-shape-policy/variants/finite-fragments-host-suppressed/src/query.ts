import { sql } from "@example/db";
const statement = sql`SELECT id FROM accounts`;
// no-mistakes-disable-next-line postgres-sql-shape-policy
statement.append(flag
  ? sql`
    WHERE pg_sleep(1) IS NULL
  ` : sql` WHERE id = 1`);
// An append-line directive also covers the SQL token inside a later branch fragment.
const invalid = sql`SELECT id FROM accounts`;
// no-mistakes-disable-next-line postgres-sql-shape-policy
invalid.append(flag
  ? sql` WHERE (`
  : sql` WHERE id = 2`);
// The invalid static branch has no original token for its synthetic SELECT origin.
