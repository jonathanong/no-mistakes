import { query, QueryExecutor, createExecutor, withTransaction, withTransactionOptions } from "@example/db";
import type { QueryExecutor as TypeExecutor } from "@example/db/types";

// Erased references must never become function-line SQL findings.
type Alias = QueryExecutor;
type Factory = typeof createExecutor;
type Named = typeof query;
query("/* named */ SELECT 1"); // CLEAR
export function direct(executor: QueryExecutor, sql: string) {
  executor("/* direct */ SELECT 1"); // CLEAR
  executor("/* offset */ SELECT 1 OFFSET 1"); // OFFSET
  executor(sql); // UNKNOWN
  const statement = "/* constant */ SELECT 1";
  executor(statement); // CLEAR
  const dynamic = sql;
  executor(dynamic); // UNKNOWN
  executor("/* missing */ SELECT 1" + sql); // OUTSIDE annotation can prove the head
  consume(executor); // ESCAPE
}
export function member(client: TypeExecutor, sql: string) {
  client.query("/* member */ SELECT 1"); // CLEAR
  client?.query("/* optional */ SELECT 1"); // CLEAR
  client.query?.("/* optional call */ SELECT 1"); // CLEAR
  client.query(sql); // UNKNOWN
}
export function destructured({ executor }: { executor: QueryExecutor }, sql: string) {
  executor("/* destructured */ SELECT 1"); // CLEAR
  executor(sql); // UNKNOWN
}
consume(QueryExecutor); // RUNTIME_TYPE
const factory = createExecutor(); // FACTORY
export function union(executor: QueryExecutor | undefined) {
  executor?.("/* union */ SELECT 1 OFFSET 1"); // UNSUPPORTED_TYPE
}
export function aliased(executor: Alias) {
  executor("/* alias */ SELECT 1 OFFSET 1"); // UNSUPPORTED_TYPE
}
// Alias cycles only preserve possible executor use; they never prove SQL.
type CycleA = QueryExecutor | CycleB;
type CycleB = CycleA;
export function cyclic(executor: CycleB) {
  consume(executor); // UNSUPPORTED_TYPE
}
export function qualified(executor: QueryExecutor.Nested) {
  executor("/* qualified */ SELECT 1"); // UNSUPPORTED_TYPE
}
export function intersection(executor: QueryExecutor & { extra: string }) {
  executor.query("/* intersection */ SELECT 1"); // UNSUPPORTED_TYPE
}
export function named(sql: string) {
  query(sql); // UNKNOWN
}
type FactoryResult = ReturnType<typeof createExecutor>;
export function factoryTyped(executor: FactoryResult, sql: string) {
  executor(sql); // UNSUPPORTED_TYPE
}
type Transaction = typeof withTransaction;
type TransactionOptions = typeof withTransactionOptions;
export function transactionTyped(executor: Transaction, sql: string) {
  executor(sql); // UNSUPPORTED_TYPE
}
export function transactionOptionsTyped(executor: TransactionOptions, sql: string) {
  consume(executor); // UNSUPPORTED_TYPE
}
// These unsupported parameter shapes retain unknown runtime uses only.
interface Derived extends QueryExecutor {}
interface UnusedDerived extends Derived {}
export function interfaceTyped(executor: Derived, sql: string) {
  executor(sql); // UNSUPPORTED_TYPE
}
export function arrayTyped([executor]: [QueryExecutor]) {
  executor("/* tuple */ SELECT 1 OFFSET 1"); // UNSUPPORTED_TYPE
}
export function restTyped(...executors: QueryExecutor[]) {
  executors[0]("/* rest */ SELECT 1"); // UNSUPPORTED_TYPE
}
export function nestedTyped({ group: { executor } }: { group: { executor: QueryExecutor } }) {
  executor("/* nested */ SELECT 1"); // UNSUPPORTED_TYPE
}
export function defaultTyped([executor = undefined, , ...others]: QueryExecutor[]) {
  executor("/* default */ SELECT 1"); // UNSUPPORTED_TYPE
  consume(others); // UNSUPPORTED_TYPE
}
export function objectRestTyped({ group: { ...executors } }: { group: { executor: QueryExecutor } }) {
  consume(executors); // UNSUPPORTED_TYPE
}
export function topRestTyped({ ...executors }: { executor: QueryExecutor }) {
  consume(executors); // UNSUPPORTED_TYPE
}
function consume(value: unknown) {}
