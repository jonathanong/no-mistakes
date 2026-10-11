import { query, sql } from "@example/db";
let statement = sql`SELECT id FROM accounts`;
(nullableFlag && sql`unused`)
  ?? statement.append(sql` OFFSET 1`);
query(statement);
