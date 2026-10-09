import sql from "sql-template-strings";
import { write } from "@app/db";

function lengthOnly(statement) {
  const ignored = opaque(arguments.length);
  return arguments[0];
}
write(lengthOnly(sql`/* primitive length */ SELECT 1`));

function computedLength(statement) {
  const ignored = opaque(arguments["length"]);
  return arguments[0];
}
write(computedLength(sql`/* computed primitive length */ SELECT 1`));

function deletedBuilder(statement) {
  delete arguments[0];
  const ignored = opaque(arguments);
  return statement;
}
// A removed slot cannot expose the still-live outer builder alias.
write(deletedBuilder(sql`/* disconnected builder */ SELECT 1`));

function maybeDeletedBuilder(statement) {
  const deletion = flag ? delete arguments[0] : false;
  const ignored = opaque(arguments);
  return statement;
}
// One branch can still expose the mutable builder through its argument slot.
write(maybeDeletedBuilder(sql`/* possibly connected builder */ SELECT 1`)); // unanalyzable:opaque-effect

const retained = sql`/* disconnected callback capture */ SELECT 1`;
function deletedCallback(callback) {
  delete arguments[0];
  const ignored = opaque(arguments);
}
const discarded = deletedCallback(() => retained.append("SELECT 2"));
write(retained);

const genericCapture = sql`/* ordinary object member */ SELECT 1`;
const object = { callback: () => genericCapture.append("SELECT 2") };
const invoked = opaque(object.callback);
// Ordinary object member projection must retain possible callbacks.
write(genericCapture); // unanalyzable:opaque-effect
