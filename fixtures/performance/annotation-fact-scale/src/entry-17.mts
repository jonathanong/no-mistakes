import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read17() {
  invoke03(() => query("/* entry_17 */ SELECT 17"), payload);
  return query("SELECT id FROM items WHERE id = 17"); // expected-finding
}
