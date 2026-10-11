import sql from "sql-template-strings";
import { write } from "@app/db";

function operandEffect() {
  write("SELECT 1"); // comparison operand effect
  return 1;
}
const inner = sql`SELECT 0`;
const comparison = inner === operandEffect();
write(sql`${comparison}`); // scalar comparison bind

// Effect-only object references still retain callbacks and captured frames.
function callbacks() {
  return cond
    ? { callback: () => write("SELECT 2") } // first object callback
    : { callback: () => write("SELECT 3") }; // second object callback
}
opaque(callbacks());

// Keeping an object opaque must not hide a builder's escape or later mutation.
const escaped = sql`/* before escape */ SELECT 4`;
const container = { escaped };
escaped.append(" WHERE active");
opaque(container);
write(escaped); // object builder escape

const assembled = sql``;
write(sql`${assembled.append("SELECT 5") === assembled}`); // mutation comparison bind
write(assembled); // interpolation mutation effect

function indexEffect() {
  write("SELECT 6"); // dynamic-index operand effect
  return 0;
}
const selectedFragment = sql`SELECT 8`;
const selected = [selectedFragment][indexEffect()];
write(sql`${selected}`); // dynamic-index fragment candidate

function objectKeyEffect() {
  write("SELECT 7"); // computed-object key effect
  return 0;
}
const keyed = { [objectKeyEffect()]: selectedFragment };
write(sql`${keyed[0]}`); // object-property fragment candidate
