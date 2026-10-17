import sql from "sql-template-strings";
import { query } from "@example/db";
import { ownerVisible, addFilter } from "./elsewhere";
import { createElement } from "./dom";

export function appended(id: string) {
  const q = sql`SELECT l.id FROM links l WHERE l.account_id = ${id} AND `;
  q.append(ownerVisible("l.parent_id"));
  q.append(sql` ORDER BY l.id`);
  query(q);
}

// An INSERT prefix is kept so the rule's INSERT fail-closed policy still sees it.
export function inserted() {
  const q = sql`INSERT INTO links (id) VALUES (1) `;
  q.append(ownerVisible("x"));
  query(q);
}

export function direct() {
  query(ownerVisible("l.parent_id"));
}

export function nonLiteral(column: string) {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible(column));
  query(q);
}

// A conditional append never composes the hole into the builder.
export function conditional(flag: boolean) {
  const q = sql`SELECT 1 WHERE `;
  if (flag) q.append(ownerVisible("x"));
  query(q);
}

export function statementBuilder(tier: string) {
  query(addFilter(sql`SELECT 1 FROM p WHERE true`, tier));
}

export function leadingHole() {
  const q = sql`${ownerVisible("x")} FROM links`;
  query(q);
}

// Aliasing a mutable builder returned by a helper stays opaque.
export function stored(tier: string) {
  const stored = addFilter(sql`SELECT 1 FROM p WHERE true`, tier);
  query(stored);
}

export function nestedFragment() {
  const inner = sql`SELECT 1 WHERE `;
  inner.append(ownerVisible("x"));
  const outer = sql`SELECT 2 `;
  outer.append(inner);
  query(outer);
}

export function alternatives(flag: boolean) {
  const text = flag ? ownerVisible("a") : "SELECT 1";
  query(text);
}

export function argumentKinds() {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible(`l.template`, 3, true, null));
  query(q);
}

// The value of an arbitrary imported call is not a SQL builder.
export function notABuilder() {
  const element = createElement("div");
  element.append(sql`SELECT 1`);
}

export function unsupportedArguments(...cols: string[]) {
  const a = sql`SELECT 1`;
  const b = sql`SELECT 2`;
  const spread = sql`SELECT 1 WHERE `;
  spread.append(ownerVisible(...cols));
  query(spread);
  const object = sql`SELECT 1 WHERE `;
  object.append(ownerVisible({ a: 1 }));
  query(object);
  const builders = sql`SELECT 1 WHERE `;
  builders.append(ownerVisible(a, b));
  query(builders);
}
