import { query } from "@example/db";

export function unchecked(table: "users" | "posts") {
  query(`SELECT id FROM ${table}`);
}
