import { write } from "@app/db";
import "./sloppy.cjs";

function strictSlotQuery() {
  const slots = arguments;
  slots[0] = "SELECT 1";
  write(slots[0]); // finding:strict-argument-alias
}

function noEscapeAlias() {
  const slots = arguments;
  slots[0] = "SELECT 2";
  write(slots[0]); // finding:no-escape-argument-alias
}

function priorEscapeAlias() {
  const slots = arguments;
  opaque(arguments);
  slots[0] = "SELECT 3";
  write(slots[0]); // finding:prior-escape-argument-alias
}

function ordinaryObject(statement) {
  const target = { 0: statement };
  target[0] = "replacement";
  return statement;
}

strictSlotQuery("unused");
noEscapeAlias("unused");
priorEscapeAlias("unused");
write(ordinaryObject(sql`/* ordinary receiver */ SELECT 1`)); // unanalyzable:ordinary-object-receiver
