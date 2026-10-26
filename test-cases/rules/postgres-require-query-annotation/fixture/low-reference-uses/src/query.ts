// Keep .ts: angle-bracket assertion targets are reserved in .mts.
import { read, sql } from "@bench/db";
declare function inspect(value: unknown): void;

export function scalarLogging() {
  let query = "SELECT 1";
  console.log(query);
  read(query); // EXPECT_MISSING_LOGGING
}

export function scalarAlias() {
  let query = "SELECT 2";
  const alias = query;
  inspect(alias);
  read(query); // EXPECT_MISSING_SCALAR_ALIAS
}

export function scalarClosureRead() {
  let query = "SELECT 3";
  const log = () => inspect(query);
  log();
  read(query); // EXPECT_MISSING_CLOSURE_READ
}

export function scalarAssignmentResult() {
  let query = "SELECT 4";
  inspect(query = "SELECT 5");
  read(query); // EXPECT_MISSING_SCALAR_RESULT
}

export function discardedTagAssignment() {
  let query = sql`/* first */ SELECT 6`;
  query = sql`/* second */ SELECT 7`;
  read(query); // EXPECT_ACCEPTED_TAG_ASSIGNMENT
}

// A used assignment result can expose the same mutable tagged builder.
export function tagAssignmentResult() {
  let query = sql`/* first */ SELECT 8`;
  inspect(query = sql`/* second */ SELECT 9`);
  read(query); // EXPECT_UNKNOWN_TAG_RESULT
}

export function tagAssignmentAlias() {
  let query = sql`/* first */ SELECT 10`;
  const alias = (query = sql`/* second */ SELECT 11`);
  inspect(alias);
  read(query); // EXPECT_UNKNOWN_TAG_RESULT_ALIAS
}

export function updatedHead() {
  let query = "/* named */ SELECT 12";
  query++;
  read(query); // EXPECT_UNKNOWN_UPDATE
}

export function shorthandTarget(record: { query: string }) {
  let query = "/* named */ SELECT 13";
  ({ query } = record);
  read(query); // EXPECT_UNKNOWN_SHORTHAND
}

export function typedTarget() {
  let query = "/* named */ SELECT 14";
  (query as string) = "SELECT 15";
  read(query); // EXPECT_UNKNOWN_TYPED_TARGET
  let nonnull = "/* named */ SELECT 19";
  nonnull! = "SELECT 20";
  read(nonnull); // EXPECT_UNKNOWN_NONNULL_TARGET
  let asserted = "/* named */ SELECT 21";
  (<string>asserted) = "SELECT 22";
  read(asserted); // EXPECT_UNKNOWN_ASSERTED_TARGET
  let satisfied = "/* named */ SELECT 23";
  (satisfied satisfies string) = "SELECT 24";
  read(satisfied); // EXPECT_UNKNOWN_SATISFIES_TARGET
}

// Unsupported destructuring order must never turn a later read into Clear.
export function defaultTarget(values: string[]) {
  let query = "/* named */ SELECT 16";
  let result: unknown;
  [query = "SELECT 17", result = read(query)] = values; // EXPECT_UNKNOWN_DEFAULT_TARGET
}

export function iterationTarget(values: string[]) {
  let query = "/* named */ SELECT 18";
  for (query of values) {
    read(query); // EXPECT_UNKNOWN_ITERATION_TARGET
  }
}
