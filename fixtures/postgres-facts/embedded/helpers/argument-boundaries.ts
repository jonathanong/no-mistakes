import sql from "sql-template-strings";
import { query } from "@example/db";
// Resolution is deferred: this fixture tests extraction of pending helper calls.
import { ownerVisible } from "./elsewhere";

// Unsupported shapes must retain only the verified prefix, never a helper hole.
export function accepted(context: { id: number }) {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible(undefined, context.id, [1, true, null, undefined], { "id": 1 }));
  query(q);
}

export function builderObject() {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible({ q }));
  query(q);
}

export function builderArray() {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible([q]));
  query(q);
}

export function nestedObject() {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible({ nested: { id: 1 } }));
  query(q);
}

export function nestedArray() {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible([[1]]));
  query(q);
}

export function spreadArray(values: unknown[]) {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible([...values]));
  query(q);
}

export function spreadObject(context: Record<string, unknown>) {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible({ ...context }));
  query(q);
}

export function computedKey(key: string) {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible({ [key]: 1 }));
  query(q);
}

export function numericKey() {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible({ 1: true }));
  query(q);
}

export function method() {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible({ id() { return 1; } }));
  query(q);
}

export function accessor() {
  const q = sql`SELECT 1 WHERE `;
  q.append(ownerVisible({ get id() { return 1; } }));
  query(q);
}

