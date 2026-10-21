import { query } from "@example/db";
import { tables } from "./tables";

export function list() {
  return query(`SELECT id FROM ${tables.users}`);
}
