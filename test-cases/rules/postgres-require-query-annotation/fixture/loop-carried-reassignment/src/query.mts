import { read, type Database } from "@example/db";
import sql from "sql-template-strings";

export function nextIteration(xs: readonly unknown[]) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    read(q); // missing: the next iteration observes the later assignment.
    q = sql`SELECT 2`;
  }
}

export function stringNextIteration(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: immutable SQL has the same backedge obligation.
    q = "SELECT 2";
  }
}

export function resetBeforeRead(xs: readonly unknown[]) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    q = sql`/* reset */ SELECT 1`;
    read(q);
    q = sql`SELECT 2`;
  }
}

export function exitsBeforeBackedge(xs: readonly unknown[]) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    read(q);
    q = sql`SELECT 2`;
    break; // There is no second executor invocation.
  }
}

export function namedBeforeBackedge(xs: readonly unknown[]) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    read(q);
    q = sql`SELECT 2`;
    q = sql`/* restored */ SELECT 3`;
  }
}

export function shadowed(xs: readonly unknown[]) {
  let q = sql`SELECT 2`;
  for (let q of xs) {
    read(q);
    q = sql`SELECT 2`;
  }
}

export function commitIsExempt(xs: readonly unknown[]) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    read(q);
    q = sql`COMMIT`;
  }
}

export function beginIsExempt(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "BEGIN";
  }
}

export function localTagShadow(xs: readonly unknown[], sql: (parts: TemplateStringsArray) => unknown) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = sql`SELECT 2`;
  }
}

export async function awaitedIteration(xs: readonly unknown[]) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    await read(q); // missing: awaiting does not erase the next iteration.
    q = sql`SELECT 2`;
  }
}

export function memberIteration(xs: readonly unknown[], client: Database) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    client.query(q); // missing: prepared receiver ownership proves this executor.
    q = sql`SELECT 2`;
  }
}

export async function awaitedMemberIteration(xs: readonly unknown[], client: Database) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    await client.query(q); // missing: both wrappers retain the executor span.
    q = sql`SELECT 2`;
  }
}

export function parameterValues(xs: readonly unknown[], client: Database, id: unknown) {
  let q = "/* named */ SELECT $1";
  for (const x of xs) {
    client.query(q, [id]); // missing: independent parameter values preserve the SQL argument.
    q = "SELECT $1";
  }
}

export async function awaitedParameterValues(xs: readonly unknown[], client: Database, id: unknown) {
  let q = "/* named */ SELECT $1";
  for (const x of xs) {
    await client.query(q, [id]); // missing: awaiting preserves parameterized SQL backedges.
    q = "SELECT $1";
  }
}

export function unsafeAdditionalBuilderReference(xs: readonly unknown[], client: Database) {
  let q = sql`/* named */ SELECT 1`;
  for (const x of xs) {
    client.query(q, [q]); // A second builder reference defeats the strict direct-use proof.
    q = sql`SELECT 2`;
  }
}

export function extraStatements(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    const label = x;
    console.log(label);
    read(q); // missing: ordinary prefix, middle, and trailing statements preserve the backedge.
    console.log(x);
    { q = "SELECT 2"; }
    if (x) console.log(label);
  }
}

export function whileIteration(again: boolean) {
  let q = "/* named */ SELECT 1";
  while (again) {
    read(q); // missing: a while backedge carries the unannotated literal.
    q = "SELECT 2";
    console.log(again);
  }
}

export function doWhileIteration(again: boolean) {
  let q = "/* named */ SELECT 1";
  do {
    console.log(again);
    read(q); // missing: do-while also carries the later literal.
    q = "SELECT 2";
  } while (again);
}

export function forInIteration(xs: Record<string, unknown>) {
  let q = "/* named */ SELECT 1";
  for (const key in xs) {
    read(q); // missing: for-in preserves the same local ownership proof.
    console.log(key);
    q = "SELECT 2";
  }
}

export function countedIteration(count: number) {
  let q = "/* named */ SELECT 1";
  for (let i = 0; i < count; i++) {
    read(q); // missing: an independent counter update cannot reset q.
    q = "SELECT 2";
    console.log(i);
  }
}

// These loops have no possible second executor invocation.
export function falseWhile() {
  let q = "/* named */ SELECT 1";
  while (false) { read(q); q = "SELECT 2"; }
}
export function falseDoWhile() {
  let q = "/* named */ SELECT 1";
  do { read(q); q = "SELECT 2"; } while (false);
}
export function returnBeforeBackedge(again: boolean) {
  let q = "/* named */ SELECT 1";
  while (again) { read(q); q = "SELECT 2"; return; }
}
export function throwBeforeBackedge(again: boolean) {
  let q = "/* named */ SELECT 1";
  do { read(q); q = "SELECT 2"; throw new Error("stop"); } while (again);
}
export function headerReset(again: boolean) {
  let q = "/* named */ SELECT 1";
  while ((q = "/* reset */ SELECT 1") && again) { read(q); q = "SELECT 2"; }
}
export function updateReset(count: number) {
  let q = "/* named */ SELECT 1";
  for (let i = 0; i < count; q = "/* reset */ SELECT 1") { read(q); q = "SELECT 2"; }
}
export function closureReset(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  const reset = () => { q = "/* reset */ SELECT 1"; };
  for (const x of xs) { read(q); q = "SELECT 2"; reset(); }
}

export function overwrittenIntermediate(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: the final missing literal supersedes an intermediate annotation.
    q = "/* intermediate */ SELECT 2";
    q = "SELECT 3";
    console.log(x);
  }
}
export function repeatedMissing(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: multiple missing writes preserve the final backedge value.
    q = "SELECT 2";
    q = "SELECT 3";
  }
}

export function explicitBackedge(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q); // missing: an unlabelled continue takes the backedge.
    q = "SELECT 2";
    continue;
    q = "/* unreachable */ SELECT 3";
  }
}
export function initializerExecutor(xs: readonly unknown[]) {
  let q = "/* named */ SELECT 1";
  for (const x of xs) {
    const rows = read(q); // missing: declaration initializers execute unconditionally.
    q = "SELECT 2";
    console.log(rows, x);
  }
}

// Function-scoped redeclarations are writes to the same binding, not shadows.
export function varResetBeforeRead(xs: readonly unknown[]) {
  var q = "/* named */ SELECT 1";
  for (const x of xs) {
    var q = "/* reset */ SELECT 2";
    read(q);
    q = "SELECT 3";
  }
}
export function varResetAfterMissing(xs: readonly unknown[]) {
  var q = "/* named */ SELECT 1";
  for (const x of xs) {
    read(q);
    q = "SELECT 2";
    var q = "/* reset */ SELECT 3";
  }
}
