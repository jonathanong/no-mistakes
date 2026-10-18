import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";
import { read } from "@example/db";

// Every read below must stay unanalyzable: the helper may or may not append,
// and the model must not pretend to know which.

// A spread of unknown options can override the earlier known `offset: false`.
function page(q: SQLStatement, opts: { offset?: boolean }) {
  const o = { offset: false, ...opts };
  if (o.offset) q.append(sql` OFFSET 10`);
}
export async function spreadOverridesDefault(opts: { offset?: boolean }) {
  const q = sql`SELECT id FROM accounts WHERE tenant_id = 1`;
  page(q, opts);
  return read(q);
}

// A hoisted nested function runs even though `return` precedes it.
function addPage(q: SQLStatement) {
  addOffset();
  return;
  function addOffset() {
    q.append(sql` OFFSET 10`);
  }
}
export async function hoistedNestedFunction() {
  const q = sql`SELECT id FROM accounts WHERE tenant_id = 1`;
  addPage(q);
  return read(q);
}

// The hoisted local shadows the module helper of the same name.
function localShadow(q: SQLStatement) {
  q.append(sql` AND tenant_id = 1`);
}
export function libShadow(q: SQLStatement) {
  localShadow(q);
  return;
  function localShadow(_q: SQLStatement) {
    _q.append(sql` OFFSET 1`);
  }
}
export async function hoistedShadowsModuleHelper() {
  const q = sql`SELECT id FROM accounts WHERE tenant_id = 1`;
  libShadow(q);
  return read(q);
}

// One builder at two parameters of a nested helper aliases them.
function both(a: SQLStatement, b: SQLStatement) {
  a.append(sql` OFFSET 10`);
  b.append(sql` AND tenant_id = 1`);
}
function outer(q: SQLStatement) {
  both(q, q);
}
export async function sameBuilderTwice() {
  const q = sql`SELECT id FROM accounts WHERE tenant_id = 1`;
  outer(q);
  return read(q);
}
