import sql from "sql-template-strings";
import { write } from "@app/db";
import sloppyReplacement from "./sloppy.cjs";

function strictReplacement(statement) {
  const ignored = (arguments[0] = "replacement");
  return statement;
}

function dynamicReplacement(statement, key) {
  const ignored = (arguments[key] = "replacement");
  return statement;
}

function deletedSlotCallback(statement) {
  delete arguments[0];
  const ignored = (arguments[0] = () => unknownMutation(statement));
  const escaped = opaque(arguments);
  return statement;
}

function deletedSlotScalar(statement) {
  delete arguments[0];
  const ignored = (arguments[0] = "replacement");
  return statement;
}

function outOfRangeSlot(statement) {
  const ignored = (arguments[1] = "replacement");
  return statement;
}

function shadowedArguments(arguments, statement) {
  const ignored = (arguments[0] = "replacement");
  return statement;
}

function opaqueRightHandSide(statement) {
  const ignored = (arguments[0] = unknownMutation(statement));
  return statement;
}

function shadowedDelete(arguments, statement) {
  delete arguments[0];
  return statement;
}

function deleteAsSlotValue(statement, other) {
  const ignored = (arguments[0] = delete arguments[1]);
  return statement;
}

write(strictReplacement(sql`/* preserved formal */ SELECT 1`)); // known:strict-formal
write(dynamicReplacement(sql`/* dynamic slot */ SELECT 1`, key)); // unanalyzable:dynamic-slot
write(sloppyReplacement(sql`/* sloppy mapped slot */ SELECT 1`)); // unanalyzable:sloppy-mapped-slot
write(deletedSlotCallback(sql`/* deleted slot callback */ SELECT 1`)); // unanalyzable:deleted-slot-callback
write(deletedSlotScalar(sql`/* deleted slot scalar */ SELECT 1`)); // known:deleted-slot-scalar
write(outOfRangeSlot(sql`/* out of range slot */ SELECT 1`)); // known:out-of-range-slot
write(shadowedArguments({}, sql`/* shadowed arguments receiver */ SELECT 1`)); // known:shadowed-arguments-receiver
write(opaqueRightHandSide(sql`/* opaque slot right hand side */ SELECT 1`)); // unanalyzable:opaque-slot-rhs
write(shadowedDelete({}, sql`/* shadowed delete receiver */ SELECT 1`)); // unanalyzable:shadowed-delete-receiver
write(deleteAsSlotValue(sql`/* delete used as slot value */ SELECT 1`, "unused")); // known:delete-slot-value
