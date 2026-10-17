import sql from "sql-template-strings";
import { query } from "@example/db";
import { ownerVisible, addFilter } from "./elsewhere";

export function afterConditional(flag: boolean, id: string) {
  const q = sql`SELECT 1 FROM t WHERE x = ${id}`;
  if (!flag) {
    q.append(sql` AND y = 2`);
  }
  q.append(ownerVisible("a.b"));
  q.append(sql` AND z = ${id}`);
  query(q);
}

export function statementHelper(flag: boolean) {
  const q = sql`SELECT 1 FROM t WHERE true`;
  if (!flag) {
    q.append(sql` AND y = 2`);
  }
  addFilter(q, "tier");
  query(q);
}

export function statementHelperInBranch(flag: boolean) {
  const q = sql`SELECT 1 FROM t WHERE true`;
  if (!flag) {
    addFilter(q, "tier");
  }
  query(q);
}

// An executor consumes the builder; it is not a helper that extends it.
export function executorIsNotAHelper() {
  const q = sql`SELECT 1 FROM t WHERE true`;
  query(q);
}
