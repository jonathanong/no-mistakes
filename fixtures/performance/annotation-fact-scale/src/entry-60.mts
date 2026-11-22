import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read60() {
  invoke03(() => query("/* entry_60 */ SELECT 60"), payload);
  return query("SELECT id FROM items WHERE id = 60"); // expected-finding
}
