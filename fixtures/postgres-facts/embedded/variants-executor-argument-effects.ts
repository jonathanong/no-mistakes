import sql from "sql-template-strings";
import { query } from "@example/db";
import { opaqueStatement } from "@example/opaque";
const gate = runtimeFlag;
const mode = runtimeMode;
const direct = sql`SELECT id FROM users`;
if (gate) direct.append(sql` WHERE id = 1`);
query(direct, direct.append(sql` OFFSET 1`)); // variants: SELECT id FROM users OFFSET 1 | SELECT id FROM users WHERE id = 1 OFFSET 1
const aliased = sql`SELECT id FROM accounts`;
if (gate) aliased.append(sql` WHERE id = 2`);
const alias = aliased;
query(aliased, alias.append(sql` OFFSET 2`)); // variants: SELECT id FROM accounts OFFSET 2 | SELECT id FROM accounts WHERE id = 2 OFFSET 2
const branched = sql`SELECT id FROM branches`;
query(branched, mode ? branched.append(sql` OFFSET 3`) : branched.append(sql` LIMIT 1`)); // variants: SELECT id FROM branches OFFSET 3 | SELECT id FROM branches LIMIT 1
const optional = sql`SELECT id FROM options`;
query(optional, mode && optional.append(sql` OFFSET 4`)); // variants: SELECT id FROM options | SELECT id FROM options OFFSET 4
let replaced = sql`SELECT id FROM original_users`;
if (gate) replaced.append(sql` WHERE id = 3`);
query(replaced, replaced = sql`SELECT id FROM replacement_users OFFSET 99`); // variants: SELECT id FROM original_users | SELECT id FROM original_users WHERE id = 3
let mixed = sql`SELECT id FROM mixed_users`;
if (gate) mixed.append(sql` WHERE id = 4`);
query(mixed, mode ? mixed.append(sql` LIMIT 1`) : (mixed = sql`SELECT 100 LIMIT 1`)); // opaque
let primitive = gate ? "SELECT 1" : "SELECT 2";
query(primitive, primitive = "SELECT 3 OFFSET 100"); // variants: SELECT 1 | SELECT 2
const escaped = sql`SELECT id FROM escaped_users`;
query(escaped, unknownHelper(escaped)); // opaque
const selected = sql`SELECT id FROM selected_users`;
query(mode ? selected : sql`SELECT 4`, selected.append(sql` OFFSET 5`)); // opaque
const copied = sql`id FROM copied_users`;
query(sql`SELECT ${copied}`, copied.append(sql` OFFSET 6`)); // variants: SELECT id FROM copied_users
const unaffected = sql`SELECT 6`;
query(unaffected, 0); // unchanged: SELECT 6
const falseAppend = sql`SELECT 7`;
query(falseAppend, false && falseAppend.append(sql` OFFSET 7`)); // unchanged: SELECT 7
const callback = sql`SELECT 8`;
query(callback, () => callback.append(sql` OFFSET 8`)); // unchanged: SELECT 8
const outer = sql`SELECT 9`;
const inner = sql`SELECT 10`;
query(outer, // unchanged: SELECT 9
  query(inner, inner.append(sql` OFFSET 9`))); // composed: SELECT 10 OFFSET 9
query("SELECT 11", direct.append(sql` OFFSET 10`)); // inline: SELECT 11
const invoked = sql`SELECT 12`;
query(invoked, (() => invoked.append(sql` OFFSET 11`))()); // opaque
const declared = sql`SELECT 13`;
query(declared, (() => { // unchanged: SELECT 13
  const unused = () => declared.append(sql` OFFSET 12`);
  return 0;
})());
const functionInvoked = sql`SELECT 14`;
query(functionInvoked, (function () { functionInvoked.append(sql` OFFSET 13`); })()); // opaque
query(opaqueStatement, 0); // opaque
query(...[direct], direct.append(sql` OFFSET 14`)); // opaque
/* A constant conditional builder has finite versions but no single object identity.
   An unknown helper receiving its alias must invalidate the original snapshot. */
const aggregate = gate ? sql`SELECT 15` : sql`SELECT 16`;
query(aggregate, aggregate.append(sql` OFFSET 15`)); // variants: SELECT 15 OFFSET 15 | SELECT 16 OFFSET 15
let uncertain = gate ? sql`SELECT 17` : sql`SELECT 18`;
query(uncertain, uncertain.append(sql` OFFSET 16`)); // opaque
const helperAggregate = gate ? sql`SELECT 19` : sql`SELECT 20`;
query(helperAggregate, unknownHelper(helperAggregate)); // opaque
const aliasAggregate = gate ? sql`SELECT 21` : sql`SELECT 22`;
const historicalAlias = aliasAggregate;
query(aliasAggregate, unknownHelper(historicalAlias)); // opaque
const isolated = gate ? sql`SELECT 23` : sql`SELECT 24`;
const unrelated = sql`SELECT 25`;
query(isolated, unknownHelper(unrelated)); // variants: SELECT 23 | SELECT 24
const stableAggregate = gate ? sql`SELECT 26` : sql`SELECT 27`;
query(stableAggregate, 0); // variants: SELECT 26 | SELECT 27
query(stableAggregate, () => unknownHelper(stableAggregate)); // variants: SELECT 26 | SELECT 27
