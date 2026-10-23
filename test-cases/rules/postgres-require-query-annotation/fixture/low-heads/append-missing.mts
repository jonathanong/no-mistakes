import { read } from "@bench/db";
import sql from "any-builder";
read(sql`SELECT 1`.append(" tail")); // head: missing
read(sql`BEGIN`.append("NING")); // head: unknown
read(sql``.append("SELECT 1")); // head: unknown
read(sql`/`.append("* name */ SELECT 1")); // head: unknown
read(sql`B`.append("EGIN")); // head: unknown
read(sql`BEGIN;`.append(" tail")); // head: clear
read(sql`BEGINNING`.append(" tail")); // head: missing
read(sql`/* named */`.append("SELECT 1")); // head: clear
read(sql`/* */ SELECT `.append("1")); // head: missing
read(sql`/* */`.append("BEGIN")); // head: unknown
function local() {
  // Later escapes cannot invalidate earlier reads, even without an append.
  const namedBeforeEscape = sql`/* named */ SELECT 1`;
  read(namedBeforeEscape); // head: clear
  stash(namedBeforeEscape);
  read(namedBeforeEscape); // head: unknown
  const missingBeforeEscape = sql`SELECT 1`;
  read(missingBeforeEscape); // head: missing
  stash(missingBeforeEscape);
  read(missingBeforeEscape); // head: unknown
  const transaction = sql`BEGIN`;
  read(transaction); // head: clear
  transaction.append("NING");
  read(transaction); // head: unknown
  const partial = sql`B`;
  read(partial); // head: missing
  partial.append("EGIN");
  read(partial); // head: unknown
  const empty = sql``;
  empty.append("/* named */ SELECT 1");
  read(empty); // head: unknown
  const slash = sql`/`;
  slash.append("* named */ SELECT 1");
  read(slash); // head: unknown
  const missing = sql`SELECT `;
  missing.append("1");
  read(missing); // head: missing
  read(missing.append(" tail")); // head: missing
  const fixed = sql`BEGIN;`;
  fixed.append(" tail");
  read(fixed); // head: clear
  const initializer = sql`SELECT `.append("1");
  read(initializer); // head: missing
  stash(initializer);
  read(initializer); // head: unknown
  const escaped = sql`SELECT `;
  stash(escaped);
  read(escaped); // head: unknown
  const captured = sql`SELECT `;
  function mutate() { captured.prepend("/* named */"); }
  read(captured); // head: unknown
  const immutable = "B";
  read(immutable); // head: missing
  read(immutable.append("EGIN")); // head: unknown
}
function repeated() {
  const transaction = sql`BEGIN`;
  for (const item of items) {
    read(transaction); // head: unknown
    transaction.append("NING");
  }
}
read("BEGINé"); // head: missing
read("BEGIN。"); // head: clear
read(`/* */ /* later */ BEGIN;${value}`); // head: clear
read(`/* */ /* later */ B${value}`); // head: unknown
