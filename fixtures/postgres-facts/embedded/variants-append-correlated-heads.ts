import { query, sql } from "@example/db";
const gate = runtimeFlag;
const q = sql`SELECT 41`;
q.append(gate ? " /* on */" : " /* off */");
// The inner use of the same immutable guard cannot contribute an opposite arm.
if (gate) q.append(gate ? " /* same */" : " /* impossible */");
query(q);
// Nested fluent links share a start span; SQL must keep both links while the
// occurrence metadata contains that physical identifier once.
query(sql`SELECT 42`.append(" /* first */").append(" /* second */"));
