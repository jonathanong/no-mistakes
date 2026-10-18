import sql from "sql-template-strings";
import { read } from "@example/db";

function mutate(statement: any) {
  statement.append(sql` OFFSET 5`);
}
const statement = sql`SELECT id FROM accounts WHERE active IS TRUE`;
mutate(statement);
// Unsupported helpers can mutate their arguments; stale complete SQL is unsafe.
read(statement); // expect: dynamic
