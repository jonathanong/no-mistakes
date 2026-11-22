import sql from "sql-template-strings";
import { query } from "@example/db";
import { lockClause } from "./lock-clause.mts";
declare function schedule(callback: () => void): void;
declare function reserveJob(): void;

export async function first() {
  const statement = sql`SELECT id FROM jobs`;
  statement.append(lockClause());
  return query(statement);
}

export async function second() {
  const statement = sql`SELECT id FROM jobs`;
  statement.append(lockClause());
  return query(statement);
}

schedule(() => {
  const statement = sql`SELECT id FROM jobs`;
  statement.append(lockClause());
  void query(statement);
});
schedule(reserveJob);
schedule(() => reserveJob());
