import { query } from "@example/db";
import { validateIdentifier as validate } from "./barrel";

export async function getter(table: string) {
  let current = "users";
  const config = { get table() { return current; } };
  validate(config.table);
  current = table;
  await query(`SELECT * FROM ${config.table}`);
}
