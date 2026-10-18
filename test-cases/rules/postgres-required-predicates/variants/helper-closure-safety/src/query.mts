import sql from "sql-template-strings";
import type { SQLStatement } from "sql-template-strings";
import { read } from "@example/db";

// The view read is appended only where the helper really runs it; a call the
// model cannot follow must stay unanalyzable instead of looking clean.

// A spread of unknown options can override the earlier known `gated: false`.
function page(q: SQLStatement, opts: { gated?: boolean }) {
  const o = { gated: false, ...opts };
  if (o.gated) q.append(sql` AND EXISTS (SELECT 1 FROM view_active_plans v WHERE v.tier = 'a')`);
}
export async function spreadOverridesDefault(opts: { gated?: boolean }) {
  const q = sql`SELECT id FROM accounts WHERE tenant_id = 1`;
  page(q, opts);
  return read(q);
}

// A hoisted nested function runs even though `return` precedes it.
function addPage(q: SQLStatement) {
  addGate();
  return;
  function addGate() {
    q.append(sql` AND EXISTS (SELECT 1 FROM view_active_plans v WHERE v.tier = 'a')`);
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
    _q.append(sql` AND EXISTS (SELECT 1 FROM view_active_plans v WHERE v.tier = 'a')`);
  }
}
export async function hoistedShadowsModuleHelper() {
  const q = sql`SELECT id FROM accounts WHERE tenant_id = 1`;
  libShadow(q);
  return read(q);
}

// One builder at two parameters of a nested helper aliases them.
function both(a: SQLStatement, b: SQLStatement) {
  a.append(sql` AND EXISTS (SELECT 1 FROM view_active_plans v WHERE v.tier = 'a')`);
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
