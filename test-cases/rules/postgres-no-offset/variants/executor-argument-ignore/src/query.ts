import sql from "sql-template-strings";
import { query } from "@example/db";
const gate = runtimeFlag;
const mode = runtimeMode;
const direct = sql`SELECT id FROM users`;
if (gate) direct.append(sql` WHERE id = 1`);
query(direct, direct.append(sql` OFFSET 1`)); // findings: offset
const branched = sql`SELECT id FROM branches`;
query(branched, mode ? branched.append(sql` OFFSET 2`) : branched.append(sql` LIMIT 1`)); // findings: offset
const optional = sql`SELECT id FROM options`;
query(optional, mode && optional.append(sql` OFFSET 3`)); // findings: offset
const escaped = sql`SELECT id FROM escaped_users`;
query(escaped, unknownHelper(escaped)); // findings: unanalyzable
let mixed = sql`SELECT id FROM mixed_users`;
if (gate) mixed.append(sql` WHERE id = 2`);
query(mixed, mode ? mixed.append(sql` LIMIT 1`) : (mixed = sql`SELECT 100 LIMIT 1`)); // findings: unanalyzable
let primitive = gate ? "SELECT 1" : "SELECT 2";
query(primitive, primitive = "SELECT 3 OFFSET 99");
let replaced = sql`SELECT id FROM original_users`;
if (gate) replaced.append(sql` WHERE id = 3`);
query(replaced, replaced = sql`SELECT id FROM replacement_users OFFSET 100`);
const invoked = sql`SELECT 4`;
query(invoked, (() => invoked.append(sql` OFFSET 4`))()); // findings: unanalyzable
/* A constant conditional builder has finite versions but no single object identity.
   An unknown helper receiving its alias must invalidate the original snapshot. */
const aggregate = gate ? sql`SELECT 5` : sql`SELECT 6`;
query(aggregate, aggregate.append(sql` OFFSET 5`)); // findings: offset
const aliasAggregate = gate ? sql`SELECT 7` : sql`SELECT 8`;
const historicalAlias = aliasAggregate;
query(aliasAggregate, unknownHelper(historicalAlias)); // findings: unanalyzable
