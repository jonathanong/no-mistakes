import sql from "sql-template-strings";
import { write } from "@app/db";

function namedProperty(statement) {
  delete arguments.extra;
  return arguments[0];
}

function computedNamedProperty(statement) {
  delete arguments["extra"];
  return arguments[0];
}

function computedNamedNonIndex(statement) {
  delete arguments[true];
  return arguments[0];
}

function nonCanonicalNumericString(statement) {
  delete arguments["01"];
  return arguments[0];
}

function fractionalNumericProperty(statement) {
  delete arguments[1.5];
  return arguments[0];
}

function selectedNumericIndex(statement, other) {
  delete arguments[1];
  return arguments[1];
}

function selectedIndex(statement, other) {
  delete arguments["1"];
  return arguments[1];
}

function preservedIndex(statement, other) {
  delete arguments["1"];
  return arguments[0];
}

function dynamicProperty(statement, key) {
  delete arguments[key];
  return arguments[0];
}

function dynamicPropertyWithKeyEffect(statement) {
  delete arguments[unknownMutation(statement)];
  return statement;
}

const namedResult = namedProperty("/* named property */ SELECT 1");
const computedNamedResult = computedNamedProperty("/* computed named property */ SELECT 1");
const nonIndexResult = computedNamedNonIndex("/* known non-index key */ SELECT 1");
const nonCanonicalResult = nonCanonicalNumericString("/* non-canonical index */ SELECT 1");
const fractionalResult = fractionalNumericProperty("/* fractional property */ SELECT 1");
const deletedIndexResult = selectedIndex(
  "/* first slot */ SELECT 1",
  "/* selected slot */ SELECT 2",
);
const deletedNumericIndexResult = selectedNumericIndex(
  "/* first numeric slot */ SELECT 1",
  "/* selected numeric slot */ SELECT 2",
);
const preservedIndexResult = preservedIndex(
  "/* retained slot */ SELECT 1",
  "/* deleted other slot */ SELECT 2",
);
const dynamicResult = dynamicProperty("/* dynamic key */ SELECT 1", "extra");
const dynamicEffectResult = dynamicPropertyWithKeyEffect(
  sql`/* dynamic key side effect */ SELECT 1`,
);

write(namedResult); // known:named-property
write(computedNamedResult); // known:computed-named-property
write(nonIndexResult); // known:known-non-index-key
write(nonCanonicalResult); // known:non-canonical-index
write(fractionalResult); // known:fractional-property
write(deletedIndexResult); // unanalyzable:deleted-index
write(deletedNumericIndexResult); // unanalyzable:deleted-numeric-index
write(preservedIndexResult); // known:preserved-index
write(dynamicResult); // unanalyzable:dynamic-property
write(dynamicEffectResult); // unanalyzable:dynamic-key-side-effect
