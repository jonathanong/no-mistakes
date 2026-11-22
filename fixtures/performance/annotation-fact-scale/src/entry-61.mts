import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read61() {
  invoke03(() => query("/* entry_61 */ SELECT 61"), payload);
  return query("SELECT id FROM items WHERE id = 61"); // expected-finding
}
