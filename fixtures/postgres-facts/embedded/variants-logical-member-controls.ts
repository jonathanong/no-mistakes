import { query } from "@example/db";
import sql from "sql-template-strings";

type Mode = "all" | "mine" | "team";

export function negatedTail(options: { mode: Mode; after?: string }) {
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
  if (options.mode !== "all" && !options.after) {
    statement.append(sql` ORDER BY id`);
  }
  query(statement);
}

export function knownMemberTruthiness(options: { mode: Mode }) {
  const statement = sql`SELECT 1 WHERE`;
  if (options.mode === "mine" && options.mode) {
    statement.append(sql` TRUE`);
  }
  if (options.mode !== "mine") {
    statement.append(sql` FALSE`);
  }
  query(statement);
}

export function sharedBinding(options: { mode: Mode }, flag: boolean) {
  const enabled = flag;
  const statement = sql`SELECT 1`;
  if (enabled) {
    statement.append(sql` ORDER BY 1`);
  }
  if (enabled && options.mode === "mine") {
    statement.append(sql` NULLS FIRST`);
  }
  query(statement);
}
