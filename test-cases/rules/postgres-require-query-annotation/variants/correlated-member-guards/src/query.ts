import { query } from "@example/db";
import sql from "sql-template-strings";

declare function observe(value: object): void;

export function directMember(options: { mode: "all" | "mine" | "team"; after?: string }) {
  const statement = sql`/* selected/read */ WITH selected AS (`;
  if (options.mode === "mine" || options.mode === "all") {
    statement.append(sql`SELECT id FROM own_items`);
  }
  if (options.mode === "all") {
    statement.append(sql` UNION ALL `);
  }
  if (options.mode === "team" || options.mode === "all") {
    statement.append(sql`SELECT id FROM team_items`);
  }
  statement.append(sql`) SELECT id FROM selected`);
  if (options.after) statement.append(sql` ORDER BY id`);
  query(statement);
}

export function aliasedMember(options: { mode: "all" | "mine" | "team"; after?: string }) {
  const mode = options.mode;
  const statement = sql`/* selected/read */ WITH selected AS (`;
  if (mode === "mine" || mode === "all") {
    statement.append(sql`SELECT id FROM own_items`);
  }
  if (mode === "all") {
    statement.append(sql` UNION ALL `);
  }
  if (mode === "team" || mode === "all") {
    statement.append(sql`SELECT id FROM team_items`);
  }
  statement.append(sql`) SELECT id FROM selected`);
  if (options.after) statement.append(sql` ORDER BY id`);
  query(statement);
}

export function receiverWrite(
  options: { mode: "all" | "mine" | "team" },
  replacement: { mode: "all" | "mine" | "team" },
) {
  options = replacement;
  const statement = sql`/* selected/read */ WITH selected AS (`;
  if (options.mode === "mine" || options.mode === "all") {
    statement.append(sql`SELECT id FROM own_items`);
  }
  if (options.mode === "all") {
    statement.append(sql` UNION ALL `);
  }
  if (options.mode === "team" || options.mode === "all") {
    statement.append(sql`SELECT id FROM team_items`);
  }
  statement.append(sql`) SELECT id FROM selected`);
  query(statement);
}

export function memberWrite(
  options: { mode: "all" | "mine" | "team" },
  replacement: "all" | "mine" | "team",
) {
  options.mode = replacement;
  const statement = sql`/* selected/read */ WITH selected AS (`;
  if (options.mode === "mine" || options.mode === "all") {
    statement.append(sql`SELECT id FROM own_items`);
  }
  if (options.mode === "all") {
    statement.append(sql` UNION ALL `);
  }
  if (options.mode === "team" || options.mode === "all") {
    statement.append(sql`SELECT id FROM team_items`);
  }
  statement.append(sql`) SELECT id FROM selected`);
  query(statement);
}

export function escapedReceiver(options: { mode: "all" | "mine" | "team" }) {
  observe(options);
  const statement = sql`/* selected/read */ WITH selected AS (`;
  if (options.mode === "mine" || options.mode === "all") {
    statement.append(sql`SELECT id FROM own_items`);
  }
  if (options.mode === "all") {
    statement.append(sql` UNION ALL `);
  }
  if (options.mode === "team" || options.mode === "all") {
    statement.append(sql`SELECT id FROM team_items`);
  }
  statement.append(sql`) SELECT id FROM selected`);
  query(statement);
}

export function openMember(options: { mode: string }) {
  const statement = sql`/* selected/read */ WITH selected AS (`;
  if (options.mode === "mine" || options.mode === "all") {
    statement.append(sql`SELECT id FROM own_items`);
  }
  if (options.mode === "all") {
    statement.append(sql` UNION ALL `);
  }
  if (options.mode === "team" || options.mode === "all") {
    statement.append(sql`SELECT id FROM team_items`);
  }
  statement.append(sql`) SELECT id FROM selected`);
  query(statement);
}

export function invalidTail(options: { mode: "all" | "mine" | "team"; after?: string }) {
  const statement = sql`/* selected/read */ WITH selected AS (`;
  if (options.mode === "mine" || options.mode === "all") {
    statement.append(sql`SELECT id FROM own_items`);
  }
  if (options.mode === "all") {
    statement.append(sql` UNION ALL `);
  }
  if (options.mode === "team" || options.mode === "all") {
    statement.append(sql`SELECT id FROM team_items`);
  }
  statement.append(sql`) SELECT id FROM selected`);
  if (options.after) statement.append(sql` ORDER BY`);
  query(statement);
}
