import sql, { default as renamed } from "sql-template-strings";
import { sql as custom } from "@example/custom";
import { read } from "@bench/db";

const tail = getSuffix();
read(sql`/* direct */ SELECT 1`.append(tail)); // low: true
read(renamed`/* named */ SELECT 1`.append(getSuffix())); // low: true
read(sql`/* chain */ SELECT 1`.append(" ").append(tail)); // low: true
read(custom`/* custom */ SELECT 1`.append(tail)); // low: false
read(sql`SELECT 1`.append(tail)); // low: false
read(sql`/* */ SELECT 1`.append(tail)); // low: false
read("/* primitive */ SELECT 1".append(tail)); // low: false
read(sql`/* spread */ SELECT 1`.append(...tail)); // low: false
read(sql`/* optional */ SELECT 1`?.append(tail)); // low: false
// Missing heads through opaque appends require the same exact tag proof as named heads.
read(custom`SELECT 1`.append(tail)); // low: false
read(unbound`SELECT 1`.append(tail)); // low: false
read("SELECT 1".append(tail)); // low: false
read(sql`/* ${name} */ SELECT 1`.append(tail)); // low: false
read(sql`B`.append(tail)); // low: false
read(sql`BEGIN`.append(tail)); // low: false
read(sql`BEGIN;`.append(tail)); // low: true

const local = sql`/* local */ SELECT 1`.append(tail);
read(local); // low: true
const escaped = sql`/* escaped */ SELECT 1`.append(tail);
change(escaped);
read(escaped); // low: false
const captured = sql`/* captured */ SELECT 1`.append(tail);
function mutate() { captured.append(" AND 1=1"); }
read(captured); // low: false
const beforeEscape = sql`/* before */ SELECT 1`.append(tail);
read(beforeEscape); // low: true
change(beforeEscape);
read(beforeEscape); // low: false

// Opaque statement appends are safe only for a local builder with a trusted root.
const statement = sql`/* statement */ SELECT 1`;
statement.append(tail);
read(statement); // low: true
read(statement.append(tail)); // low: true
const customBuilder = custom`/* custom-builder */ SELECT 1`;
customBuilder.append(tail);
read(customBuilder); // low: false
const scalar = "/* scalar */ SELECT 1";
scalar.append(tail);
read(scalar.append(tail)); // low: false
const selfMutating = sql`/* self */ SELECT 1`;
selfMutating.append(change(selfMutating));
read(selfMutating); // low: false
const capturedSuffix = sql`/* capture */ SELECT 1`;
function suffix() { capturedSuffix.append(" AND 1=1"); return tail; }
capturedSuffix.append(suffix());
read(capturedSuffix); // low: false
const helperEscape = sql`/* helper */ SELECT 1`;
change(helperEscape);
helperEscape.append(tail);
read(helperEscape); // low: false
const ordered = sql`/* ordered */ SELECT 1`;
read(ordered); // low: true
ordered.append(tail);
read(ordered); // low: true
