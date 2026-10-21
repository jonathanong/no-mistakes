import { query } from "@example/db";
import * as t from "./tables";

export function list() {
  return query(`SELECT id FROM ${t.tables.users}`);
}
