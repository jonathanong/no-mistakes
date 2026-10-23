import { read, sql } from "@bench/db";

declare function buildSql(): string;

export function branchVar(enabled: boolean) {
  if (enabled) {
    var statement = "SELECT 1";
  }
  read(statement); // EXPECT_MISSING_PRIMARY_VAR EXPECT_MAX_MISSING
}

export function reassignedLet(enabled: boolean) {
  let statement = "SELECT 2"; // EXPECT_MAX_MISSING
  if (enabled) statement = `SELECT 3`; // EXPECT_MAX_MISSING
  read(statement); // EXPECT_MISSING_PRIMARY_REASSIGNED
}

export function allNamed(enabled: boolean) {
  let statement: string;
  if (enabled) statement = "/* named/one */ SELECT 4";
  else statement = `/* named/two */ SELECT 5`;
  read(statement); // EXPECT_ACCEPTED_ALL_NAMED
}

export function mixedPaths(enabled: boolean) {
  let statement = "/* named */ SELECT 6";
  if (enabled) statement = "SELECT 7"; // EXPECT_MAX_MISSING
  read(statement); // EXPECT_MISSING_PRIMARY_MIXED
}

export function callAssignment(enabled: boolean) {
  let statement = "SELECT 8";
  if (enabled) statement = buildSql();
  read(statement); // EXPECT_UNKNOWN_CALL
}

export function undefinedPath(enabled: boolean) {
  let statement: string | undefined;
  if (enabled) statement = "/* named */ SELECT 9";
  read(statement); // EXPECT_ACCEPTED_UNDEFINED
}

export function closureWrite(enabled: boolean) {
  let statement = "SELECT 10";
  const update = () => { statement = "/* named */ SELECT 10"; };
  if (enabled) update();
  read(statement); // EXPECT_UNKNOWN_CLOSURE
}

export function parameterHead(statement: string) {
  read(statement); // EXPECT_UNKNOWN_PARAMETER
}

export function aliasMutation() {
  let statement = sql`/* named */ SELECT 11`;
  const alias = statement;
  alias.append(" /* a possible alias write */");
  read(statement); // EXPECT_UNKNOWN_ALIAS_MUTATION
}

export function shadowed() {
  let statement = "SELECT 12";
  {
    let statement = "/* inner */ SELECT 13";
    read(statement); // EXPECT_ACCEPTED_SHADOW
  }
  read(statement); // EXPECT_MISSING_OUTER_SHADOW
}

export function writeOrdering() {
  let statement = "SELECT 14";
  read(statement); // EXPECT_MISSING_BEFORE_WRITE
  statement = buildSql();
  read(statement); // EXPECT_UNKNOWN_AFTER_WRITE
}

export function lineSuppression() {
  let statement = "SELECT 15";
  // no-mistakes-disable-next-line postgres-require-query-annotation
  read(statement); // EXPECT_SUPPRESSED_ASSIGNMENT
}

// Undefined is neutral only when its reference is not shadowed.
export function explicitUndefined(enabled: boolean) {
  let statement = undefined;
  if (enabled) statement = "/* named */ SELECT 16";
  read(statement); // EXPECT_ACCEPTED_EXPLICIT_UNDEFINED
}

export function shadowedUndefined(undefined: string, enabled: boolean) {
  let statement = undefined;
  if (enabled) statement = "/* named */ SELECT 17";
  read(statement); // EXPECT_UNKNOWN_SHADOWED_UNDEFINED
}

export function voidPath(enabled: boolean) {
  let statement = void 0;
  if (enabled) statement = `SELECT 18`;
  read(statement); // EXPECT_MISSING_VOID_PATH
}

// A textual later write can reach the next iteration's earlier call.
export function repeatedUnknown(enabled: boolean) {
  let statement = "/* named */ SELECT 19";
  while (enabled) {
    read(statement); // EXPECT_UNKNOWN_LOOP_WRITE
    statement = buildSql();
  }
}

export function compoundAssignment() {
  let statement = "/* named */ SELECT 20";
  statement += " WHERE active";
  read(statement); // EXPECT_UNKNOWN_COMPOUND
}

export function iterationTarget(values: string[]) {
  let statement = "/* named */ SELECT 21";
  for (statement of values) {
    read(statement); // EXPECT_UNKNOWN_ITERATION_TARGET
  }
}

export function destructuredAssignment(values: string[]) {
  let statement = "/* named */ SELECT 22";
  [statement] = values;
  read(statement); // EXPECT_UNKNOWN_DESTRUCTURE
}

export function localTaggedLiteral() {
  let statement = sql`/* named */ SELECT 23`;
  read(statement); // EXPECT_ACCEPTED_MUTABLE_TAG
}

// Deferred class bodies must not inherit their enclosing function's proof.
export function classContexts() {
  let statement = "/* named */ SELECT 24";
  class Box {
    value = read(statement); // EXPECT_UNKNOWN_CLASS_FIELD
    accessor slot = read(statement); // EXPECT_UNKNOWN_ACCESSOR
    static { read(statement); } // EXPECT_UNKNOWN_STATIC_BLOCK
  }
}

export function countedAndRepeated(enabled: boolean, values: string[]) {
  for (let statement = "SELECT 25"; enabled;) {
    read(statement); // EXPECT_MISSING_FOR_LITERAL
  }
  for (var key in values) {
    read(key); // EXPECT_UNKNOWN_FOR_IN_DECLARATION
  }
  for (let value of values) {
    read(value); // EXPECT_UNKNOWN_FOR_OF_DECLARATION
  }
  let repeated = "SELECT 26";
  do {
    read(repeated); // EXPECT_MISSING_DO_LITERAL
    repeated = "/* named */ SELECT 27";
  } while (enabled);
}

export function effectfulVoid() {
  let statement = void buildSql();
  read(statement); // EXPECT_UNKNOWN_EFFECTFUL_VOID
}

export function destructuredHead(record: { statement: string }) {
  let { statement } = record;
  read(statement); // EXPECT_UNKNOWN_DESTRUCTURED_HEAD
}

declare function acquire(): any;
export function resourceHead() {
  using statement = acquire();
  read(statement); // EXPECT_UNKNOWN_USING_HEAD
}
