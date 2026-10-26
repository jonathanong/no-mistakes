import { read, type QueryExecutor, type QueryOptions } from "@example/db";
import SQL from "sql-template-strings";

// Executor-bearing aggregates become parameter data only at a checked SQL sink.
function consumeBound(query: QueryExecutor, payload: unknown) {
  query(SQL`SELECT ${payload}`); // CLEAR_BIND_BODY
}
consumeBound(read, { executor: read }); // CLEAR_OBJECT_BIND_TRANSPORT
consumeBound(read, [read]); // CLEAR_ARRAY_BIND_TRANSPORT

// A returned unsupported member cannot certify nonexecution of a carrier.
function escapedMember(query: QueryExecutor) {
  return query.other.nested;
}
read(escapedMember(read)); // ADVISE_MEMBER_CARRIER_ESCAPE

// An opaque optional receiver remains uncertain; a typed fallback cannot prove every branch.
export function optionalReceiver(options: QueryOptions, fallback: QueryExecutor) {
  const query = options.query ?? fallback;
  const statement = SQL`SELECT 1`;
  statement.append(SQL` WHERE true`);
  query(statement); // ADVISE_OPTIONAL_LOCAL_BUILDER
}

// Direct configured query-member destructuring supplies an exact replay receiver.
export function destructuredReceiver({ query }: QueryOptions) {
  const statement = SQL`SELECT 1`;
  statement.append(SQL` WHERE true`);
  query(statement); // CLEAR_DESTRUCTURED_LOCAL_BUILDER
}

declare const patchUnknownChoice: boolean;

// A returned aggregate still carries its executor; it is not a nonexecuting proof.
function returnExecutorObject(query: QueryExecutor) {
  return { query };
}
function returnExecutorArray(query: QueryExecutor) {
  return [query];
}
read(returnExecutorObject(read)); // ADVISE_RETURNED_OBJECT_CARRIER
read(returnExecutorArray(read)); // ADVISE_RETURNED_ARRAY_CARRIER

// Nested live builders cannot cross the bounded read-only helper boundary.
function readOnlyArguments(value: unknown, executor: QueryExecutor) {
  return "SELECT 1";
}
read(readOnlyArguments({ statement: SQL`SELECT 1` }, read)); // ADVISE_OBJECT_BUILDER_ARGUMENT
read(readOnlyArguments([SQL`SELECT 1`], read)); // ADVISE_ARRAY_BUILDER_ARGUMENT
function returnBuilderObject() {
  return { statement: SQL`SELECT 1` };
}
read(returnBuilderObject()); // ADVISE_RETURNED_BUILDER_OBJECT
read({ statement: SQL`SELECT 1` }); // ADVISE_ROOT_BUILDER_OBJECT
read([SQL`SELECT 1`]); // ADVISE_ROOT_BUILDER_ARRAY

// These direct SQL arguments reach the actual append evaluator without helper lookup.
read("SELECT 1".append(SQL` WHERE true`)); // ADVISE_DIRECT_APPEND_RECEIVER
read(SQL`SELECT 1`.append(null)); // ADVISE_DIRECT_APPEND_SUFFIX

// Logical retention must preserve executor payload in constrained/nullable alternatives.
function retainedExecutorMember(query: QueryExecutor) {
  const retained = query.query || query;
  return retained.other;
}
function nullishExecutorMember(query: QueryExecutor) {
  const retained = query.query ?? query;
  return retained.other;
}
read(retainedExecutorMember(read)); // ADVISE_CONSTRAINED_EXECUTOR_MEMBER
read(nullishExecutorMember(read)); // ADVISE_NULLISH_EXECUTOR_MEMBER

// The corresponding scalar nullish alternatives remain harmless and analyzable.
function scalarNullish(value: unknown) {
  return (value ?? {}).field ? "SELECT 1" : "SELECT 2";
}
read(scalarNullish(patchUnknownChoice)); // CLEAR_SCALAR_NULLISH_MEMBER
