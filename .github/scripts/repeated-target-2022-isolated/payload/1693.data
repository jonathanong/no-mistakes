import sql, { type SQLStatement } from "sql-template-strings";
import { query } from "@example/db";
const gate = runtimeFlag;
function withOffset(input: SQLStatement, label: string): SQLStatement {
  return input.append(sql` OFFSET 7`);
}
const existing = sql`SELECT id FROM users`;
const alias = existing;
const unrelated = sql`SELECT id FROM accounts`;
query(gate ? withOffset(existing, "first") : sql`SELECT 2`);
query(existing);
query(alias);
query(unrelated);
query(gate ? unrelated : sql`SELECT 3`);
const shadowBuilder = sql`SELECT id FROM orders`;
{
  function withOffset(shadowInput: SQLStatement, label: string): SQLStatement {
    return shadowInput;
  }
  query(gate ? withOffset(shadowBuilder, "shadow") : sql`SELECT 4`);
}
query(shadowBuilder);
let reassigned = sql`SELECT id FROM old_users`;
const previous = reassigned;
query(withOffset(reassigned, (reassigned = sql`SELECT id FROM new_users`)));
query(previous);
query(gate ? reassigned : sql`SELECT 5`);
const incomplete = sql`SELECT id FROM incomplete_users`;
query(gate ? withOffset(incomplete) : sql`SELECT 6`);
query(incomplete);
query(withOffset(sql`SELECT id FROM recent_users`, "fluent").append(sql` LIMIT 2`));
function passthrough(pureInput: SQLStatement): SQLStatement {
  return pureInput;
}
const pure = sql`SELECT id FROM pure_users`;
const pureAlias = pure;
query(passthrough(pure));
query(gate ? pure : sql`SELECT 7`);
query(gate ? pureAlias : sql`SELECT 8`);
var redeclared = sql`SELECT id FROM redeclared_users`;
const redeclaredAlias = redeclared;
query(gate ? redeclaredAlias : sql`SELECT 9`);
// A declaration without an initializer preserves the original mutable object.
var redeclared;
query(withOffset(redeclared, "var"));
query(gate ? redeclaredAlias : sql`SELECT 10`);
const appendBase = sql`SELECT id FROM append_users`;
const appendAlias = appendBase.append(sql` WHERE active`);
query(gate ? appendBase : sql`SELECT 11`);
query(withOffset(appendAlias, "stored append"));
query(gate ? appendBase : sql`SELECT 11`);
const passthroughBase = sql`SELECT id FROM passthrough_users`;
const passthroughAlias = passthrough(passthroughBase);
query(gate ? passthroughBase : sql`SELECT 12`);
query(withOffset(passthroughAlias, "stored passthrough"));
query(gate ? passthroughBase : sql`SELECT 12`);
const nestedParent = sql`SELECT id FROM nested_users`;
query(gate ? nestedParent : sql`SELECT 13`);
withOffset(passthrough(nestedParent), "nested input");
query(nestedParent);
const otherParent = sql`SELECT id FROM other_users`;
query(gate ? otherParent : sql`SELECT 14`);
withOffset(otherParent.append(sql` LIMIT 1`), "append input");
query(otherParent);
const firstParent = sql`SELECT id FROM first_users`;
const secondParent = sql`SELECT id FROM second_users`;
query(gate ? firstParent : sql`SELECT 15`);
query(gate ? secondParent : sql`SELECT 16`);
withOffset(gate ? firstParent : secondParent, "conditional input");
query(firstParent);
query(secondParent);
