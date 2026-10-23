import { sql } from "@example/db"; import { type SQLStatement as Statement } from "sql-template-strings"; /* Configured tags splice fragments; sql-template-strings binds them. */
import { query } from "@example/db";
type SQLStatement = string;
function appendAlias(fragment: Statement): Statement {
  const returned = fragment.append(sql` FROM users`);
  return returned;
}
query(sql`${appendAlias(sql`SELECT id`)}`);
query(sql`${appendAlias(runtimeValue)}`);
async function asyncFragment(promiseFragment: Statement): Promise<Statement> {
  return promiseFragment;
}
query(sql`SELECT ${asyncFragment(sql`SELECT 3`)}`);
function scalarResult(scalarInput: Statement): number {
  scalarInput.append(sql` FROM accounts`);
  return 42;
}
query(sql`SELECT ${scalarResult(sql`SELECT 4`)}`);
function untrustedType(untrustedInput: SQLStatement): SQLStatement {
  return untrustedInput;
}
query(sql`SELECT ${untrustedType("SELECT 5")}`);
