import { query } from "@example/db";
import sql from "sql-template-strings";

export function listEither(mode: "widgets" | "gadgets" | "all") {
  const statement = sql`/* listEither */ WITH candidates AS (`;
  if (mode === "widgets" || mode === "all") {
    statement.append(sql`SELECT id FROM widgets`);
  }
  if (mode === "all") {
    statement.append(sql` UNION ALL `);
  }
  if (mode === "gadgets" || mode === "all") {
    statement.append(sql`SELECT id FROM gadgets`);
  }
  statement.append(sql`) SELECT id FROM candidates LIMIT 5`);
  query(statement);
}

export function independent(left: "widgets" | "gadgets", right: "widgets" | "gadgets") {
  const statement = sql`SELECT 1`;
  if (left === "widgets") statement.append(sql` /* widgets */`);
  if (right === "gadgets") statement.append(sql` /* gadgets */`);
  query(statement);
}

export function reassigned(mode: "widgets" | "gadgets", replacement: typeof mode) {
  const statement = sql`SELECT 1`;
  if (mode === "widgets") statement.append(sql` /* widgets */`);
  mode = replacement;
  if (mode === "gadgets") statement.append(sql` /* gadgets */`);
  query(statement);
}

export function nested(mode: "widgets" | "gadgets" | "all") {
  const statement = sql`SELECT 1`;
  if ("all" !== mode && !(mode === "gadgets" || false)) {
    statement.append(sql` /* widgets */`);
  } else {
    statement.append(sql` /* gadgets */`);
  }
  if (mode === "all") statement.append(sql` /* all */`);
  query(statement);
}

export function openString(mode: string) {
  const statement = sql`SELECT 1`;
  if (mode === "widgets") statement.append(sql` /* widgets */`);
  if (mode === "gadgets") statement.append(sql` /* gadgets */`);
  query(statement);
}

export function numeric(mode: 1 | 2) {
  const statement = sql`SELECT 1`;
  if (mode === 1) statement.append(sql` /* widgets */`);
  if (mode === 2) statement.append(sql` /* gadgets */`);
  query(statement);
}

export function opaque(mode: "widgets" | "gadgets", enabled: boolean) {
  const statement = sql`SELECT 1`;
  if (mode === "widgets" && enabled) statement.append(sql` /* widgets */`);
  if (mode === "gadgets") statement.append(sql` /* gadgets */`);
  query(statement);
}

export function objectOperand(mode: "widgets" | "gadgets") {
  const options = { table: "widgets" };
  const statement = sql`SELECT 2`;
  if (options === mode) statement.append(sql` /* branch */`);
  query(statement);
}

export function arrayOperand(mode: "widgets" | "gadgets") {
  const options = ["widgets"];
  const statement = sql`SELECT 2`;
  if (options === mode) statement.append(sql` /* branch */`);
  query(statement);
}

export function overPairBudget(
  left: "a" | "b" | "c" | "d" | "e",
  right: "f" | "g" | "h" | "i" | "j",
) {
  const statement = sql`SELECT 2`;
  if (left === right) statement.append(sql` /* branch */`);
  query(statement);
}

export function overCombinedPathBudget(
  left: "a" | "b" | "c" | "d" | "e",
  right: "f" | "g" | "h" | "i" | "j",
) {
  const statement = sql`SELECT 2`;
  if (left === "a" || left === "b" || left === "c" || left === "d" || left === "e") {
    if (left === right) statement.append(sql` /* branch */`);
    query(statement);
  }
}

export function overDepthBudget(mode: "a" | "b") {
  const statement = sql`SELECT 2`;
  if (
    mode === "a" &&
    (true && (true && (true && (true && (true && (true && (true && (true && (true && mode === "b")))))))))
  ) {
    statement.append(sql` /* branch */`);
  }
  query(statement);
}
