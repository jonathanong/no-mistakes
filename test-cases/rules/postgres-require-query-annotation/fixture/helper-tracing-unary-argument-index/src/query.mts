import { write } from "@app/db";

function readsPositiveZero(statement) {
  write(arguments[+0]); // finding:positive-zero-read-is-canonical-index-zero
}
readsPositiveZero("SELECT 1");

function readsNegativeZero(statement) {
  write(arguments[-0]); // finding:negative-zero-read-is-canonical-index-zero
}
readsNegativeZero("SELECT 2");

function writesPositiveZero(statement) {
  arguments[+0] = "SELECT 3";
  write(arguments[+0]); // finding:positive-zero-write-is-canonical-index-zero
}
writesPositiveZero("/* input annotation */ SELECT 1");

function writesNegativeZero(statement) {
  arguments[-0] = "SELECT 4";
  write(arguments[-0]); // finding:negative-zero-write-is-canonical-index-zero
}
writesNegativeZero("/* input annotation */ SELECT 1");

function readsNestedPositiveZero(statement) {
  write(arguments[+(+0)]); // finding:nested-positive-zero-read
}
readsNestedPositiveZero("SELECT 5");

function readsNestedNegativeZero(statement) {
  write(arguments[-(-0)]); // finding:nested-negative-zero-read
}
readsNestedNegativeZero("SELECT 6");

function writesNestedPositiveZero(statement) {
  arguments[+(+0)] = "SELECT 7";
  write(arguments[+(+0)]); // finding:nested-positive-zero-write
}
writesNestedPositiveZero("/* input annotation */ SELECT 1");

function writesNestedNegativeZero(statement) {
  arguments[-(-0)] = "SELECT 8";
  write(arguments[-(-0)]); // finding:nested-negative-zero-write
}
writesNestedNegativeZero("/* input annotation */ SELECT 1");

function readsCanonicalStringZero(statement) {
  write(arguments["0"]); // finding:canonical-string-zero
}
readsCanonicalStringZero("SELECT 9");

// These values deliberately are not canonical nonnegative integer indices.
function nonCanonicalIndexShapes(statement, key) {
  const negative = arguments[-1];
  const fractional = arguments[1.5];
  const unsafe = arguments[9007199254740992];
  const leadingZero = arguments["01"];
  const named = arguments["extra"];
  const coercive = arguments[+key];
  const nestedCoercive = arguments[+(+key)];
  const nestedNonNumeric = arguments[+!key];
  const dynamic = arguments[key];
  const logicalNot = arguments[!key];
  return [
    negative,
    fractional,
    unsafe,
    leadingZero,
    named,
    coercive,
    nestedCoercive,
    nestedNonNumeric,
    dynamic,
    logicalNot,
  ];
}
nonCanonicalIndexShapes("SELECT 10", "0");
