import { query } from "@example/db";
import { tables } from "./tables";

export function list(id: string) {
  return query(`SELECT id FROM users WHERE name = ${tables.users} AND id = ${id}`);
}
