import { read } from "@bench/db";
import sql from "sql-template-strings";
// Unknown tails stay unanalyzable even when the head is already named.
read(sql`/* direct */ SELECT 1`.append(dynamic)); // low: false
read(sql`/* empty */ SELECT 1`.append()); // low: false
read(sql`/* extra */ SELECT 1`.append("", dynamic)); // low: false
read(sql`/* spread */ SELECT 1`.append(...parts)); // low: false
read(sql`/* untagged */ SELECT 1`.append(` WHERE x = ${value}`)); // low: false
read(sql`/* tagged */ SELECT 1`.append(sql` WHERE x = ${value}`)); // low: true
read(sql`/* nested */ SELECT 1`.append(dynamic).append(" tail")); // low: false
export function bound() {
  const q = sql`/* local */ SELECT 1`;
  q.append(sql` WHERE x = ${value}`);
  read(q); // low: true
  q.append(dynamic);
  read(q); // low: false
}
export function boundSpread() {
  const q = sql`/* spread */ SELECT 1`;
  q.append(...parts);
  read(q); // low: false
}
export function initializer() {
  const q = sql`/* initialized */ SELECT 1`.append(dynamic);
  read(q); // low: false
}
export function conditional() {
  const q = sql`/* branches */ SELECT 1`;
  if (condition) q.append(sql` WHERE x = ${value}`);
  for (const item of items) q.append(" AND TRUE");
  read(q); // low: true
}
export function plainStringAppend() {
  read("/* plain */ SELECT 1".append(dynamic)); // low: false
}
