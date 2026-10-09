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

function recreateDenseCallbackAfterEscape(statement) {
  const escaped = opaque(arguments);
  delete arguments[0];
  const ignored = (arguments[0] = () => unknownMutation(statement));
  return statement;
}

function createSparseCallbackAfterEscape(statement) {
  const escaped = opaque(arguments);
  delete arguments[9];
  const ignored = (arguments[9] = () => unknownMutation(statement));
  return statement;
}

function overwriteLiveDenseSlotAfterEscape(statement) {
  const escaped = opaque(arguments);
  const ignored = (arguments[0] = () => unknownMutation(statement));
  return statement;
}

function createSparseSlotAfterEscape(statement) {
  const escaped = opaque(arguments);
  const ignored = (arguments[9] = () => unknownMutation(statement));
  return statement;
}

function installDenseLiteralCallbackAfterEscape(statement, second) {
  const escaped = opaque(arguments);
  const ignored = (arguments[1] = () => write("SELECT 1")); // finding:dense-literal-callback-after-escape
  return statement;
}

function installSparseLiteralCallbackAfterEscape(statement) {
  const escaped = opaque(arguments);
  const ignored = (arguments[9] = () => write("SELECT 1")); // finding:sparse-literal-callback-after-escape
  return statement;
}

function invokeNestedDenseLiteralInstaller(statement) {
  function install(second) {
    const escaped = opaque(arguments);
    const ignored = (arguments[0] = () => write("SELECT 1")); // finding:nested-dense-literal-callback-after-escape
  }
  install("unused");
  return statement;
}

function invokeNestedSparseLiteralInstaller(statement) {
  function install() {
    const escaped = opaque(arguments);
    const ignored = (arguments[9] = () => write("SELECT 1")); // finding:nested-sparse-literal-callback-after-escape
  }
  install();
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

function outOfRangeStoredValue(statement) {
  const ignored = (arguments[9] = statement);
  return statement;
}

function extraBuilderAppend(statement) {
  const ignored = (arguments[9] = statement);
  statement.append(" /* changed after extra-slot storage */");
  return statement;
}

function extraSlotRead(statement) {
  const ignored = (arguments[9] = statement);
  return arguments[9];
}

function selectedWritePreservesOtherSlot(statement, other) {
  const ignored = (arguments[1] = "replacement");
  return arguments[0];
}

function priorEscapeRemainsTainted(statement, other) {
  const escaped = opaque(arguments);
  const ignored = (arguments[1] = "replacement");
  return arguments[0];
}

function extraCallbackNoEscape(statement) {
  const ignored = (arguments[9] = () => unknownMutation(statement));
  return statement;
}

function extraCallbackEscapes(statement) {
  const ignored = (arguments[9] = () => unknownMutation(statement));
  const escaped = opaque(arguments);
  return statement;
}

function deletedExtraCallback(statement) {
  const ignored = (arguments[9] = () => unknownMutation(statement));
  delete arguments[9];
  return statement;
}

function recreatedDeletedExtraCallback(statement) {
  delete arguments[9];
  const ignored = (arguments[9] = () => unknownMutation(statement));
  return statement;
}

function hugeExtraSlot(statement) {
  const ignored = (arguments[9007199254740991] = statement);
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

async function awaitedStrictDelete(statement) {
  await delete arguments[0];
  return statement;
}

function writeRecreatedDeletedProperty(statement) {
  delete arguments[0];
  const ignored = (arguments[0] = "SELECT 1");
  write(arguments[0]); // finding:recreated-deleted-property
  return statement;
}

write(strictReplacement(sql`/* preserved formal */ SELECT 1`)); // known:strict-formal
write(dynamicReplacement(sql`/* dynamic slot */ SELECT 1`, key)); // unanalyzable:dynamic-slot
write(sloppyReplacement(sql`/* sloppy mapped slot */ SELECT 1`)); // unanalyzable:sloppy-mapped-slot
write(deletedSlotCallback(sql`/* deleted slot callback */ SELECT 1`)); // unanalyzable:deleted-slot-callback
write(recreateDenseCallbackAfterEscape(sql`/* dense callback after escape */ SELECT 1`)); // unanalyzable:dense-callback-after-escape
write(createSparseCallbackAfterEscape(sql`/* sparse callback after escape */ SELECT 1`)); // unanalyzable:sparse-callback-after-escape
write(overwriteLiveDenseSlotAfterEscape(sql`/* live dense callback after escape */ SELECT 1`)); // unanalyzable:live-dense-callback-after-escape
write(createSparseSlotAfterEscape(sql`/* live sparse callback after escape */ SELECT 1`)); // unanalyzable:live-sparse-callback-after-escape
write(installDenseLiteralCallbackAfterEscape(sql`/* dense literal callback */ SELECT 1`, "unused")); // unanalyzable:dense-literal-callback-after-escape
write(installSparseLiteralCallbackAfterEscape(sql`/* sparse literal callback */ SELECT 1`)); // unanalyzable:sparse-literal-callback-after-escape
write(invokeNestedDenseLiteralInstaller(sql`/* nested dense literal callback */ SELECT 1`)); // unanalyzable:nested-dense-literal-callback-after-escape
write(invokeNestedSparseLiteralInstaller(sql`/* nested sparse literal callback */ SELECT 1`)); // unanalyzable:nested-sparse-literal-callback-after-escape
write(deletedSlotScalar(sql`/* deleted slot scalar */ SELECT 1`)); // known:deleted-slot-scalar
write(outOfRangeSlot(sql`/* out of range slot */ SELECT 1`)); // known:out-of-range-slot
write(outOfRangeStoredValue(sql`/* out of range stored value */ SELECT 1`)); // known:out-of-range-stored-value
write(extraBuilderAppend(sql`/* extra builder append */ SELECT 1`)); // known:extra-builder-append
write(extraSlotRead(sql`/* extra slot read */ SELECT 1`)); // known:extra-slot-read
write(selectedWritePreservesOtherSlot(sql`/* selected slot write */ SELECT 1`, "unused")); // known:selected-slot-write
write(priorEscapeRemainsTainted(sql`/* prior escape taint */ SELECT 1`, "unused")); // unanalyzable:prior-escape-taint
write(extraCallbackNoEscape(sql`/* extra callback no escape */ SELECT 1`)); // known:extra-callback-no-escape
write(extraCallbackEscapes(sql`/* extra callback escapes */ SELECT 1`)); // unanalyzable:extra-callback-escapes
write(deletedExtraCallback(sql`/* deleted extra callback */ SELECT 1`)); // known:deleted-extra-callback
write(recreatedDeletedExtraCallback(sql`/* recreated deleted extra callback */ SELECT 1`)); // known:recreated-deleted-extra-callback
write(hugeExtraSlot(sql`/* bounded huge extra slot */ SELECT 1`)); // known:huge-extra-slot
write(shadowedArguments({}, sql`/* shadowed arguments receiver */ SELECT 1`)); // known:shadowed-arguments-receiver
write(opaqueRightHandSide(sql`/* opaque slot right hand side */ SELECT 1`)); // unanalyzable:opaque-slot-rhs
write(shadowedDelete({}, sql`/* shadowed delete receiver */ SELECT 1`)); // unanalyzable:shadowed-delete-receiver
write(deleteAsSlotValue(sql`/* delete used as slot value */ SELECT 1`, "unused")); // known:delete-slot-value
write(await awaitedStrictDelete(sql`/* awaited strict delete */ SELECT 1`)); // known:awaited-strict-delete
writeRecreatedDeletedProperty(sql`/* original property */ SELECT 1`);
