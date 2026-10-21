import { query } from "@example/db";
import * as tables from "./tables";

export function list() {
  return query(`SELECT id FROM ${tables.tables.users}`);
}
