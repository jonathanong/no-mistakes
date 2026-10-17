import sql from "sql-template-strings";
import { query } from "@example/db";
import type { typeOnly } from "./a";
import { type inlineType, value, renamed as alias } from "./a";
import * as namespace from "./b";
import defaultHelper from "./c";

export function typeOnlyCall() {
  const q = sql`SELECT 1 WHERE `;
  q.append(typeOnly());
  query(q);
}

export function inlineTypeCall() {
  const q = sql`SELECT 1 WHERE `;
  q.append(inlineType());
  query(q);
}

export function valueCall() {
  const q = sql`SELECT 1 WHERE `;
  q.append(value());
  query(q);
}

export function aliasCall() {
  const q = sql`SELECT 1 WHERE `;
  q.append(alias());
  query(q);
}

export function namespaceCall() {
  const q = sql`SELECT 1 WHERE `;
  q.append(namespace.member());
  query(q);
}

export function defaultCall() {
  const q = sql`SELECT 1 WHERE `;
  q.append(defaultHelper());
  query(q);
}

// A parameter named like the import shadows it.
export function shadowed(value: () => string) {
  const q = sql`SELECT 1 WHERE `;
  q.append(value());
  query(q);
}
