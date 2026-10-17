import { query, sql } from "@example/db";
const gate = runtimeFlag;
const otherGate = otherRuntimeFlag;
const statement = sql`SELECT id FROM users`;
const alias = statement;
if (gate) statement.append(sql` OFFSET 7`); // findings: offset
query(gate ? alias : sql`SELECT 2`); // findings: unanalyzable
query(gate ? statement : sql`SELECT 2`);
const computed = sql`SELECT id FROM users`;
const computedAlias = computed;
computed["append"](sql` OFFSET 8`);
query(gate ? computedAlias : sql`SELECT 3`); // findings: unanalyzable
query(gate ? computed : sql`SELECT 4`); // findings: unanalyzable
const fluent = sql`SELECT id FROM users`;
const fluentAlias = fluent;
fluent.append(sql` WHERE id = 1`).append(sql` OFFSET 9`);
query(gate ? fluentAlias : sql`SELECT 5`); // findings: unanalyzable
query(gate ? fluent : sql`SELECT 6`); // findings: unanalyzable
let replaced = sql`SELECT id FROM users`;
const previous = replaced;
replaced = sql`SELECT id FROM accounts`;
if (gate) replaced.append(sql` OFFSET 10`); // findings: offset
query(gate ? previous : sql`SELECT 7`);
query(gate ? replaced : sql`SELECT 8`);
let uncertain = sql`SELECT id FROM users`;
const oldAlias = uncertain;
if (gate) uncertain = sql`SELECT id FROM accounts`;
uncertain.append(sql` OFFSET 11`); // findings: offset
query(otherGate ? oldAlias : sql`SELECT 9`); // findings: unanalyzable
query(otherGate ? uncertain : sql`SELECT 10`);
let shadowed = sql`SELECT id FROM users`;
const shadowAlias = shadowed;
{
  const shadowed = shadowAlias;
  if (gate) shadowed.append(sql` OFFSET 12`);
}
query(otherGate ? shadowed : sql`SELECT 11`); // findings: unanalyzable
query(otherGate ? shadowAlias : sql`SELECT 12`); // findings: unanalyzable
let text = "SELECT id FROM users";
const copied = text;
if (gate) text += " OFFSET 13"; // findings: offset
query(otherGate ? copied : "SELECT 14");
query(gate ? text : "SELECT 15");
const initial = sql`SELECT id FROM users`;
const immediateAlias = initial.append(sql` LIMIT 1`);
query(gate ? immediateAlias : sql`SELECT 16`);
query(immediateAlias);
const base = sql`SELECT id FROM users`;
const through = base;
if (gate) through.append(sql` OFFSET 14`); // findings: offset
query(otherGate ? base : sql`SELECT 17`); // findings: unanalyzable
query(gate ? through : sql`SELECT 18`);
