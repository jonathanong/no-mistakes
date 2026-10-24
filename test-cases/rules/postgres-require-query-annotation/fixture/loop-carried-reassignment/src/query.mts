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
