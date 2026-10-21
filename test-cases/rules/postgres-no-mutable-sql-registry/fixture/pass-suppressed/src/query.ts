import { query } from "@example/db";
import { tables } from "./tables";

export function list() {
  // no-mistakes-disable-next-line postgres-no-mutable-sql-registry
  return query(`SELECT id FROM ${tables.users}`);
}
