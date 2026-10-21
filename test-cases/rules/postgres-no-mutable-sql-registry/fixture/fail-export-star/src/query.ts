import { query } from "@example/db";
import { tables } from "./index";

export function list() {
  return query(`SELECT id FROM ${tables.users}`);
}
