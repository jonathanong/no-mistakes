import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read07() {
  invoke03(() => query("/* entry_07 */ SELECT 7"), payload);
  return query("SELECT id FROM items WHERE id = 7"); // expected-finding
}
