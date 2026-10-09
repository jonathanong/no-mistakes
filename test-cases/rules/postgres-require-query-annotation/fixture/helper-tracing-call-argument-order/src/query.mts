import sql from "sql-template-strings";
import { write } from "@app/db";

function forward(first, ignored) {
  return first;
}
async function promised(first) {
  return first;
}

// Earlier operands hold object identities, not pre-mutation SQL snapshots.
const directBuilder = sql``;
const directResult = forward(directBuilder, directBuilder.append("SELECT 1"));
write(directResult); // finding:missing-annotation

const wrappedBuilder = sql``;
const wrappedResult = forward((0, wrappedBuilder), wrappedBuilder.append("SELECT 1"));
write(wrappedResult); // finding:missing-annotation

const promiseBuilder = sql``;
const promiseResult = forward(promised(promiseBuilder), promiseBuilder.append("SELECT 1"));
write(await promiseResult); // finding:missing-annotation

const nestedBuilder = sql``;
const nestedResult = forward([nestedBuilder], nestedBuilder.append("SELECT 1"));
write(nestedResult[0]); // unanalyzable:array-projection

const annotatedBuilder = sql``;
const annotatedResult = forward(annotatedBuilder, annotatedBuilder.append("/* later annotation */ SELECT 1"));
write(annotatedResult);

// A primitive string argument is an immutable copy, even when another builder changes.
const copyBuilder = sql``;
const immutableResult = forward("/* literal copy */ SELECT 1", copyBuilder.append("SELECT 2"));
write(immutableResult);

const escapedBuilder = sql`/* escaped later operand */ SELECT 1`;
const escapedResult = forward(escapedBuilder, opaque(escapedBuilder));
write(escapedResult); // unanalyzable:opaque-effect

const nestedEscapedBuilder = sql`/* nested escaped later operand */ SELECT 1`;
const nestedEscapedResult = forward([nestedEscapedBuilder], opaque(nestedEscapedBuilder));
write(nestedEscapedResult[0]); // unanalyzable:opaque-effect
