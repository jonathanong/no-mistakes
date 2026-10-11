import { sql } from "@example/db";
const unsafe = sql` WHERE pg_sleep(1) IS NULL`;
const allowed = sql` WHERE id = 1`;
const statement = sql`SELECT id FROM accounts`;
statement.append(flag ? unsafe : allowed);
statement.append(otherFlag ? unsafe : allowed);
// This builder is never executed, and both appends share the same unsafe SQL token.
const invalid = sql`SELECT id FROM accounts`;
invalid.append(flag
  ? sql` WHERE (`
  : sql` WHERE id = 2`);
// Without a host directive, invalid SQL keeps its branch fragment's existing line.
