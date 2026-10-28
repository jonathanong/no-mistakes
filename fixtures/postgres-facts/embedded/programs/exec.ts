import { sql } from "@example/db"; /* Configured tags splice fragments; sql-template-strings binds them. */
import type { SQLStatement } from "sql-template-strings";

// Fresh builders and strings.
export function fresh(id: string) {
  return sql`SELECT ${id}`;
}
export function rawSplice(alias: string) {
  return `${alias}.x IS NULL`;
}
export function literalString() {
  return "a = 1";
}
export function concatenated(a: string, b: string) {
  return a + " AND " + b;
}
export function numbers(a: string) {
  const n = 1;
  return n + a;
}

// Builder parameters.
export function appender(q: SQLStatement) {
  q.append(sql` AND a = 1`);
}
export function returnsBuilder(q: SQLStatement) {
  q.append(sql` AND b = 1`);
  return q;
}
export function readsBuilderParameter(q: SQLStatement) {
  return sql`SELECT 1 WHERE `.append(q);
}
export function interpolatesBuilderParameter(q: SQLStatement) {
  return sql`SELECT ${q}`;
}
export function appendsToNonBuilder(q: string) {
  q.append(sql` AND a = 1`);
}
export function appendsNumber(q: SQLStatement) {
  q.append(1);
}
export function interpolatesFreeName() {
  return sql`SELECT ${notDeclared}`;
}
export function inlinesFragment(q: SQLStatement, id: string) {
  const inner = sql`b = ${id}`;
  q.append(sql` AND `).append(inner);
}
export function aliasOfParameter(q: SQLStatement) {
  const alias = q;
  alias.append(sql` AND c = 1`);
  return alias;
}

// Tests and forks.
export function branches(flag: boolean) {
  const q = sql`SELECT 1`;
  if (flag) {
    q.append(sql` WHERE a = 1`);
  } else {
    q.append(sql` WHERE b = 1`);
  }
  return q;
}
export function repeatedTest(options: { x: boolean }) {
  const q = sql`SELECT 1`;
  if (options.x) q.append(sql` AND a = 1`);
  if (options.x) q.append(sql` AND b = 1`);
  if (!options.x) q.append(sql` AND c = 1`);
  return q;
}
export function negatedTests(options: { x?: boolean; y?: string }) {
  const q = sql`SELECT 1`;
  if (options.x !== true) q.append(sql` AND a = 1`);
  if (options.x === true) q.append(sql` AND b = 1`);
  if (options.y != null) q.append(sql` AND c = 1`);
  if (options.y == "z") q.append(sql` AND d = 1`);
  return q;
}
export function logicalTests(a: boolean, b: boolean) {
  const q = sql`SELECT 1`;
  if (a && b) q.append(sql` AND both = 1`);
  if (a || b) q.append(sql` AND either = 1`);
  return q;
}
export function valueLogic(a, b) {
  const x = a && b;
  const y = a || b;
  const z = a ?? b;
  return sql`SELECT ${x}, ${y}, ${z}`;
}
export function statementLogic(flag: boolean, q: SQLStatement) {
  flag && q.append(sql` AND a = 1`);
  flag ? q.append(sql` AND b = 1`) : q.append(sql` AND c = 1`);
}
export function ternaryText(flag: boolean, alias: string) {
  return sql`SELECT 1 WHERE `.append(flag ? `${alias}.a = 1` : `${alias}.b = 1`);
}
export function throwsOnUnsafe(alias: string) {
  if (alias === "bad") throw new Error("unsafe");
  return sql`SELECT 1`;
}
export function alwaysThrows() {
  throw new Error("never returns");
}
export function earlyReturn(flag: boolean) {
  const q = sql`SELECT 1`;
  if (flag) {
    return q;
  }
  q.append(sql` WHERE a = 1`);
  return q;
}
export function nestedBlock(flag: boolean) {
  const q = sql`SELECT 1`;
  {
    const inner = flag ? " WHERE a = 1" : " WHERE b = 1";
    q.append(inner);
  }
  return q;
}
export function withDefault(alias: string, strict = true, label = alias + "!") {
  const q = sql`SELECT 1`;
  if (strict) q.append(sql` WHERE strict = 1`);
  q.append(` /* ${label} */`);
  return q;
}

// Objects, arrays and members.
export function members(options?: { a?: string; nested?: { b: string } }) {
  const q = sql`SELECT 1`;
  q.append(` /* ${options?.a} */`);
  return q;
}
export function memberOfLiteral(config: { table: string }) {
  return sql`SELECT 1 FROM `.append(config.table);
}
export function memberOfUnknown(config) {
  return sql`SELECT ${config.id}, ${config["name"]}`;
}
export function memberOfString(text: string) {
  return sql`SELECT ${text.length}`;
}
export function spreads(base: { a: string }) {
  const merged = { ...base, b: "x" };
  return sql`SELECT 1`.append(merged.b);
}
export function spreadsNothing(base) {
  const merged = { ...null, ...undefined, ...base, c: "z" };
  return sql`SELECT 1`.append(merged.c);
}
export function spreadsString() {
  const merged = { ..."text" };
  return sql`SELECT 1`;
}
export function computedKey() {
  const key = "a";
  const o = { [key]: "x" };
  return sql`SELECT 1`;
}
export function arrayLength(items: string[]) {
  return sql`SELECT ${items.length}`;
}
export function arrayElementsAreBound(items: string[]) {
  return sql`SELECT ${items}`;
}
export function sumOver(tables: string[]) {
  const q = sql`SELECT 1`;
  for (const table of tables) {
    q.append(sql` UNION SELECT 1 FROM `).append(table);
  }
  return q;
}
export function returnsInsideLoop(tables: string[]) {
  for (const table of tables) {
    return sql`SELECT 1 FROM `.append(table);
  }
  return sql`SELECT 0`;
}
export function loopOverUnknown(items) {
  for (const item of items) {
    return sql`SELECT 1`;
  }
  return sql`SELECT 0`;
}

// Calls.
export function callsFresh(id: string) {
  return sql`WITH x AS (`.append(fresh(id)).append(sql`)`);
}
export function callsAppender(q: SQLStatement) {
  appender(q);
  return q;
}
export function callsReturningBuilder(q: SQLStatement) {
  const same = returnsBuilder(q);
  return same;
}
export function callsUnresolvedStatement(alias: string) {
  audit(alias);
  return sql`SELECT 1`;
}
export function callsUnresolvedValue(alias: string) {
  return sql`SELECT ${audit(alias)}`;
}
export function callsUnresolvedWithObject(q: SQLStatement) {
  audit({ q });
  return sql`SELECT 1`;
}
export function passesObjectsAndArrays(alias: string) {
  return takesShapes({ a: alias, list: ["x", "y"], none: null, gone: undefined, on: true });
}
export function takesShapes(options: { a: string; list: string[]; none: null; gone; on: boolean }) {
  return sql`SELECT 1`.append(options.a);
}
export function passesBuilderInObject(q: SQLStatement) {
  takesShapes({ q });
  return q;
}
export function callsThroughParameter(build: () => SQLStatement) {
  return build();
}
export function callsFailing() {
  return fails();
}
export function fails() {
  return sql`SELECT ${alwaysFails}`;
}
export function selfRecursive() {
  return selfRecursive();
}

// Statements outside the model.
export function usesAssignment(q: SQLStatement) {
  let text = "a";
  text = "b";
  return sql`SELECT 1`.append(text);
}
export function usesTry() {
  try {
    return sql`SELECT 1`;
  } catch (error) {
    return sql`SELECT 2`;
  }
}
export function usesSwitch(mode: string) {
  switch (mode) {
    case "a":
      return sql`SELECT 1`;
  }
  return sql`SELECT 2`;
}
export function usesVar() {
  var q = sql`SELECT 1`;
  return q;
}
export function usesDestructuring(source) {
  const { a } = source;
  return sql`SELECT 1`;
}
export function usesAwait() {
  return sql`SELECT 1`;
}
export function usesTemplateTag() {
  return other`SELECT 1`;
}
export function usesRawTag() {
  return String.raw`SELECT 1`;
}
export function untaken(flag: boolean) {
  if (flag) {
    q = unknownGlobal;
  }
  return sql`SELECT 1`;
}
export function opaqueTaken() {
  q = unknownGlobal;
  return sql`SELECT 1`;
}
export function emptyReturn() {
  return;
}
export function voidBody(q: SQLStatement) {
  q.append(sql` AND a = 1`);
}

// More values, forks and failures.
export function numberBinds(id: string) {
  return sql`SELECT ${1}, ${id}`;
}
export function addsBuilder() {
  return "a" + sql`b`;
}
export function addsFree() {
  return "a" + notDeclared;
}
export function addsUnknown(x: string) {
  const v = x + "a";
  return sql`SELECT ${v}`;
}
export function arrayMember(items: string[]) {
  return sql`SELECT ${items.first}`;
}
export function thirtyTwoPaths(o: Record<string, boolean>) {
  const q = sql`SELECT 1`;
  if (o.a) q.append(sql` AND a = 1`);
  if (o.b) q.append(sql` AND b = 1`);
  if (o.c) q.append(sql` AND c = 1`);
  if (o.d) q.append(sql` AND d = 1`);
  if (o.e) q.append(sql` AND e = 1`);
  return q;
}
export function loopThatForks(items: string[], o) {
  const q = sql`SELECT 1`;
  for (const item of items) {
    if (o.flag === item) q.append(sql` AND a = 1`);
  }
  return q;
}
export function forkingDefault(flag: boolean, x = flag ? "a" : "b") {
  return sql`SELECT 1`.append(x);
}
export function nullish(a) {
  const v = a ?? "x";
  return sql`SELECT 1`.append(v);
}
export function negationValue(flag) {
  const v = !flag;
  return sql`SELECT ${v}`;
}
export function equalityValue(a, b) {
  const v = a === b;
  return sql`SELECT ${v}`;
}
export function callsWithFree() {
  return fresh(notDeclared);
}
export function passesOpenObject(base) {
  return takesShapes({ ...base, a: "x" });
}
export function passesNestedBuilder(q: SQLStatement) {
  takesShapes({ list: [q] });
  return sql`SELECT 1`;
}

// Literal switch arms. `default` must throw, or the helper stays opaque.
export function markReadSql(kind: string) {
  switch (kind) {
    case "post":
      return sql`UPDATE posts SET read_at = NOW() WHERE id = ${1}`;
    case "comment":
      return sql`UPDATE comments SET read_at = NOW() WHERE id = ${1}`;
    default:
      throw new Error("unknown type");
  }
}
// #2203 rewrite: one complete statement per arm, not correlated appends.
export function accessSql(access: string) {
  switch (access) {
    case "owned":
      return sql`SELECT id FROM orders WHERE account_id = ${1}`;
    case "member":
      return sql`SELECT id FROM orders WHERE member_id = ${1}`;
    case "all":
      return sql`SELECT id FROM orders WHERE account_id = ${1} UNION ALL SELECT id FROM orders WHERE member_id = ${1}`;
    default:
      throw new Error("unknown access");
  }
}
// #2198 rewrite: literal identifiers, not a record lookup.
export function tableName(kind: string) {
  switch (kind) {
    case "x":
      return "orders";
    case "y":
      return "accounts";
    default:
      throw new Error("unknown table");
  }
}
export function groupedKind(kind: string) {
  switch (kind) {
    case "a":
    case "b":
      return sql`SELECT 1 FROM posts WHERE id = ${1}`;
    case "c":
      return sql`SELECT 1 FROM comments WHERE id = ${1}`;
    default: {
      throw new Error("unknown kind");
    }
  }
}

// A parameter read into a fresh const cannot change the final throw.
export function constThenThrow(kind: string) {
  switch (kind) {
    case "x":
      return sql`CREATE TABLE finite_x (id int)`;
    case "y":
      return sql`SELECT 1`;
    default: {
      const unexpected: never = kind as never;
      throw new Error(String(unexpected));
    }
  }
}
function abrupt(value: string): never {
  throw new Error(value);
}
export function callToAbrupt(kind: string) {
  switch (kind) {
    case "x":
      return sql`CREATE TABLE finite_x (id int)`;
    default:
      return abrupt(kind);
  }
}
function canReturn(value: string) { return value; }
export function callCanReturn(kind: string) {
  switch (kind) {
    case "x":
      return sql`CREATE TABLE finite_x (id int)`;
    default:
      return canReturn(kind);
  }
}
async function asyncAbort(value: string): Promise<never> { throw new Error(value); }
function* generatorAbort(value: string) { throw new Error(value); }
let mutableAbort = (value: string): never => { throw new Error(value); };
mutableAbort = value => value as never;
export function unsafeDefaults(kind: string) {
  switch (kind) {
    case "x": return sql`CREATE TABLE finite_x (id int)`;
    default: return asyncAbort(kind);
  }
}
export function generatorDefault(kind: string) {
  switch (kind) {
    case "x": return sql`CREATE TABLE finite_x (id int)`;
    default: return generatorAbort(kind);
  }
}
export function optionalDefault(kind: string) {
  switch (kind) {
    case "x": return sql`CREATE TABLE finite_x (id int)`;
    default: return abrupt?.(kind);
  }
}
export function reassignedDefault(kind: string) {
  switch (kind) {
    case "x": return sql`CREATE TABLE finite_x (id int)`;
    default: return mutableAbort(kind);
  }
}
function failClosed(value: never): never {
  throw new Error(String(value));
}
export function tableViaThrow(kind: string) {
  switch (kind) {
    case "x":
      return "orders";
    case "y":
      return "accounts";
    default:
      failClosed(kind);
  }
}
// The name is not proof. The body returns, so the switch stays opaque.
function assertNever(value: string): string {
  return value;
}
export function assertNeverIsNotProof(kind: string) {
  switch (kind) {
    case "x":
      return sql`SELECT 1 FROM orders`;
    default:
      return assertNever(kind);
  }
  return sql`SELECT 2`;
}
// A parameter shadows the proven thrower, so the default is not proof.
export function shadowedThrowParam(kind, failClosed) {
  switch (kind) {
    case "a":
      return sql`SELECT 1`;
    default:
      failClosed(kind);
  }
  return sql`SELECT 2`;
}
function throwsViaShadowedParam(k, failClosed) {
  return failClosed(k);
}
// The wrapper's default calls that shadowed body, which is not proof either.
export function shadowedThrowWrapper(kind) {
  switch (kind) {
    case "a":
      return sql`SELECT 1`;
    default:
      throwsViaShadowedParam(kind, failClosed);
  }
  return sql`SELECT 2`;
}
// Duplicate labels abstain the switch. A known arm is not kept.
export function duplicateLabel(kind: string) {
  switch (kind) {
    case "a":
      return sql`SELECT 1`;
    case "a":
      return sql`SELECT 2`;
    default:
      throw new Error("no");
  }
  return sql`SELECT 3`;
}
// The first "a" falls into default. The later "a" is not a separate arm.
export function duplicateAfterDefault(kind: string) {
  switch (kind) {
    case "a":
    default:
      throw new Error("no");
    case "a":
      return sql`SELECT * FROM a19 OFFSET 1`;
    case "b":
      return sql`SELECT * FROM b19 OFFSET 1`;
  }
}
// A local before the return is not the literal-return shape.
export function switchWithLocal(kind: string) {
  switch (kind) {
    case "a": {
      const extra = "x";
      return sql`SELECT ${extra}`;
    }
    default:
      throw new Error("no");
  }
  return sql`SELECT 2`;
}
// Seventeen arms: the cap abstains instead of keeping sixteen.
export function switchCap(kind: string) {
  switch (kind) {
    case "01": return sql`SELECT 1`;
    case "02": return sql`SELECT 1`;
    case "03": return sql`SELECT 1`;
    case "04": return sql`SELECT 1`;
    case "05": return sql`SELECT 1`;
    case "06": return sql`SELECT 1`;
    case "07": return sql`SELECT 1`;
    case "08": return sql`SELECT 1`;
    case "09": return sql`SELECT 1`;
    case "10": return sql`SELECT 1`;
    case "11": return sql`SELECT 1`;
    case "12": return sql`SELECT 1`;
    case "13": return sql`SELECT 1`;
    case "14": return sql`SELECT 1`;
    case "15": return sql`SELECT 1`;
    case "16": return sql`SELECT 1`;
    case "17": return sql`SELECT 1`;
    default:
      throw new Error("no");
  }
}
export async function asyncSwitch(kind: string) {
  switch (kind) {
    case "a":
      return sql`SELECT 1`;
    default:
      throw new Error("no");
  }
}

// These member calls lack a source-proven purity contract in helper lowering.
export function guardedLimit(limit: number) {
  if (!Number.isInteger(limit)) throw new Error("bad limit");
  return sql`SELECT id FROM orders WHERE account_id = ${1} LIMIT ${limit}`;
}
export function guardedBlock(limit: number) {
  if (!Number.isInteger(limit)) {
    throw new Error("bad limit");
  }
  return sql`SELECT id FROM orders WHERE account_id = ${1}`;
}
// `&&` cannot make opaque member calls safe to omit.
export function guardedAnd(limit: number, offset: number) {
  if (!Number.isInteger(limit) && !Number.isInteger(offset)) throw new Error("bad");
  return sql`SELECT id FROM orders WHERE account_id = ${1}`;
}
// The opaque method may mutate a builder even when its guard does not throw.
export function mutatingMethodGuard(guard: { update(q: unknown): boolean }) {
  const q = sql`SELECT 1`;
  if (!guard.update(q)) throw new Error("bad");
  return q;
}
// Not a throw guard: both arms return SQL, and the test is opaque.
export function opaqueBranch(limit: number) {
  if (!Number.isInteger(limit)) {
    return sql`SELECT 0`;
  }
  return sql`SELECT 1`;
}

// `??` on a bound value stays a bind. The constant is not SQL text.
const START = "0";
export function nullishOuter(after) {
  const afterId = after ?? START;
  return sql`SELECT id FROM orders WHERE account_id = ${1} AND id > ${afterId}`;
}
export function nullishLocal(after) {
  const start = "0";
  const afterId = after ?? start;
  return sql`SELECT id FROM orders WHERE id > ${afterId}`;
}
// The left side is an unknown fragment: do not publish the fallback.
export function nullishFragment() {
  return missing ?? sql`SELECT 1`;
}
export function nullishSqlFallback(a) {
  return a ?? sql`SELECT 1`;
}
export function nullishNestedFallback(x) {
  return sql`SELECT * FROM n14 WHERE a = ${x ?? sql`SELECT * FROM n14b OFFSET 4`} OFFSET 5`;
}

export function looseEquality(a, b) {
  if (a == b) return sql`yes`;
  return sql`no`;
}
export function strictEquality(a, b) {
  if (a === b) return sql`yes`;
  return sql`no`;
}
