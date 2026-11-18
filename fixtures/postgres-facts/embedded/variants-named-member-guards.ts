import { query } from "@example/db";
import sql from "sql-template-strings";

type Mode = "all" | "mine" | "team";

export function mixedTail(options: { mode: Mode; after?: string }) {
  const statement = sql`WITH selected AS (`;
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
  if (options.mode !== "all" && options.after) {
    statement.append(sql` ORDER BY id`);
  }
  query(statement);
}

export function separateMembers(options: { mode: Mode; limitMode: Mode }) {
  const statement = sql`WITH selected AS (`;
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
  if (options.limitMode === "mine") {
    statement.append(sql` LIMIT 1`);
  }
  if (options.limitMode === "team") {
    statement.append(sql` LIMIT 2`);
  }
  query(statement);
}
