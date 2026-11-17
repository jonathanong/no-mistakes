import { query } from "@example/db";
import { sql as rawSql } from "@example/raw-sql";
import { getConfig } from "./test-helpers/shallow";

export function run() {
  getConfig("vote").table = "audit";
  const config = getConfig("vote");
  return query(rawSql`SELECT id FROM ${config.table}`);
}
