import { query, QueryExecutor, createExecutor, withTransaction, withTransactionOptions } from "@example/db";
import type { QueryExecutor as TypeExecutor } from "@example/db/types";
import type { OtherExecutor } from "@example/db";

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
const factory = createExecutor(); // CLEAR factory transport executes no statement
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
// Interface member dependencies retain the same unknowns as object aliases.
interface MemberContext { executor: QueryExecutor }
export function memberContext(context: MemberContext, sql: string) {
  context.executor(sql); // UNSUPPORTED_TYPE
}
function consume(value: unknown) {}
// Generic constraints/defaults preserve uncertainty, not literal proofs.
export function constrained<T extends QueryExecutor>(executor: T, sql: string) {
  executor("/* generic */ SELECT 1 OFFSET 1"); // UNSUPPORTED_TYPE
}
type DefaultExecutor<T = QueryExecutor> = T;
export function genericDefault(executor: DefaultExecutor, sql: string) {
  executor(sql); // UNSUPPORTED_TYPE
}
interface DefaultContext<T = QueryExecutor> { executor: T }
export function interfaceDefault(context: DefaultContext, sql: string) {
  context.executor(sql); // UNSUPPORTED_TYPE
}
// Non-parameter annotations preserve runtime uncertainty, never literal proof.
declare function unconfigured(): QueryExecutor;
const annotatedLocal: QueryExecutor = unconfigured();
annotatedLocal("/* local */ SELECT 1 OFFSET 1"); // UNSUPPORTED_TYPE
export class FieldRunner {
  db!: QueryExecutor;
  run() { this.db.query("/* field */ SELECT 1 OFFSET 1"); } // UNKNOWN
}
export class ParameterRunner {
  constructor(private db: QueryExecutor) {}
  run() { this.db.query("/* parameter property */ SELECT 1 OFFSET 1"); } // UNKNOWN
}
// A same-spelled field in another class and dynamic this are unrelated.
export class OtherRunner {
  db!: unknown;
  run() { this.db.query("SELECT 1 OFFSET 1"); } // CLEAR
}
export class ScopedRunner {
  db!: QueryExecutor;
  static run() { this.db.query("SELECT 1 OFFSET 1"); } // CLEAR
  method() {
    function dynamicThis() { this.db.query("SELECT 1 OFFSET 1"); } // CLEAR
    const arrow = () => this.db.query("SELECT 1 OFFSET 1"); // UNKNOWN
    class Nested { run() { this.db.query("SELECT 1 OFFSET 1"); } } // CLEAR
    return arrow;
  }
}
type ErasedField = typeof FieldRunner.prototype.db;
export function shadowedLocal() {
  type QueryExecutor = { query(sql: string): unknown };
  const local: QueryExecutor = unconfigured();
  local.query("SELECT 1 OFFSET 1"); // CLEAR
}
export class MemberShapes {
  db!: QueryExecutor;
  #db!: QueryExecutor;
  ordinary = 1;
  constructor(ordinary: string) {}
  field = this.db; // UNSUPPORTED_TYPE
  static { consume(this.db); } // UNSUPPORTED_TYPE
  static db: QueryExecutor = unconfigured();
  static run() { this.db.query("SELECT 1 OFFSET 1"); } // UNSUPPORTED_TYPE
  run() {
    this[unconfigured()].query("SELECT 1 OFFSET 1"); // CLEAR
    this['db'].query("SELECT 1 OFFSET 1"); // UNKNOWN
    this.#db.query("SELECT 1 OFFSET 1"); // UNSUPPORTED_TYPE
  }
}
declare function makeBase(value: unknown): any;
export class EnclosingTyped {
  db!: QueryExecutor;
  run() {
    class Inner extends makeBase(this.db) { db!: unknown; } // UNSUPPORTED_TYPE
    class Keyed { [consume(this.db)]() {} } // UNSUPPORTED_TYPE
  }
}
export class EnclosingUntyped {
  db!: unknown;
  run() {
    class Inner extends makeBase(this.db) { db!: QueryExecutor; } // CLEAR
    class Keyed { db!: QueryExecutor; [consume(this.db)]() {} } // CLEAR
  }
}
export class AccessorRunner {
  accessor db: QueryExecutor = unconfigured();
  accessor saved = this.db; // UNSUPPORTED_TYPE
  run() { this.db.query("SELECT 1 OFFSET 1"); } // UNSUPPORTED_TYPE
}
export class PrivateIdentity {
  #db!: QueryExecutor;
  '#db'!: unknown;
  run() { this['#db'].query("SELECT 1 OFFSET 1"); } // CLEAR
}
// An erased class index signature never becomes a runtime executor use.
export class ErasedIndexSignature { [key: string]: QueryExecutor; } // CLEAR
// A parameter property without an annotation supplies no executor type proof.
export class InferredProperty { constructor(private db = unconfigured()) {} } // CLEAR
// Nested runtime obligations keep their own runtime use lines.
annotatedLocal( // UNSUPPORTED_TYPE
  annotatedLocal("/* nested local */ SELECT 1 OFFSET 1"), // UNSUPPORTED_TYPE
);
// An unknown enclosing or nested SQL call never covers a distinct receiver.
export function nestedUnconfigured(other: OtherExecutor, sql: string) {
  query( // UNKNOWN
    other.query(sql), // FALLBACK_UNKNOWN
  );
  other.query( // FALLBACK_UNKNOWN
    query(sql), // UNKNOWN
  );
}
// Return annotations stay erased; only their own executing value is uncertain.
declare function unconfiguredAny(): any; // CLEAR
function getDb(): QueryExecutor { // CLEAR
  return unconfiguredAny(); // UNSUPPORTED_TYPE
}
const returnedDb = getDb(); // CLEAR
returnedDb.query("SELECT 1 OFFSET 1"); // CLEAR
const assertedDb = unconfiguredAny() as QueryExecutor; // UNSUPPORTED_TYPE
assertedDb.query("SELECT 1 OFFSET 1"); // CLEAR
const angleDb = <QueryExecutor>unconfiguredAny(); // UNSUPPORTED_TYPE
angleDb.query("SELECT 1 OFFSET 1"); // CLEAR
const satisfiedDb = unconfiguredAny() satisfies QueryExecutor; // UNSUPPORTED_TYPE
satisfiedDb.query("SELECT 1 OFFSET 1"); // CLEAR
const objectDb = {
  db: null as any as QueryExecutor, // UNSUPPORTED_TYPE
  run() { this.db.query("SELECT 1 OFFSET 1"); }, // CLEAR
};
objectDb.run(); // CLEAR
// Nested functions own their return obligation; enclosing annotations do not leak.
function outerTyped(): QueryExecutor { // CLEAR
  function innerUntyped() { return unconfiguredAny(); } // CLEAR
  innerUntyped(); // CLEAR
  return unconfiguredAny(); // UNSUPPORTED_TYPE
}
function outerUntyped() { // CLEAR
  function innerTyped(): QueryExecutor { // CLEAR
    return unconfiguredAny(); // UNSUPPORTED_TYPE
  }
  innerTyped(); // CLEAR
  return unconfiguredAny(); // CLEAR
}
const arrowTyped = (): QueryExecutor => unconfiguredAny(); // UNSUPPORTED_TYPE
const blockArrowTyped = (): QueryExecutor => { // CLEAR
  return unconfiguredAny(); // UNSUPPORTED_TYPE
};
class ReturningMethod {
  getDb(): QueryExecutor { // CLEAR
    return unconfiguredAny(); // UNSUPPORTED_TYPE
  }
}
type ErasedReturn = ReturnType<typeof getDb>; // CLEAR
// A return annotation and transparent assertion describe one runtime value.
function assertedReturn(): QueryExecutor { // CLEAR
  return (unconfiguredAny() as QueryExecutor); // UNSUPPORTED_TYPE
}
const objectProducer = { query: unconfiguredAny() } satisfies { query: QueryExecutor }; // UNSUPPORTED_TYPE
