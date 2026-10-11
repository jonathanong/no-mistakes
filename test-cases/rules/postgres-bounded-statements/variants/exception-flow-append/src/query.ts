import sql from "sql-template-strings";
import { query } from "@example/db";
const statement = sql`SELECT id FROM accounts`;
try {
  mayThrow();
  statement.append(sql` LIMIT 1`);
} catch {
  query(statement);
} finally {
  query(statement);
}
