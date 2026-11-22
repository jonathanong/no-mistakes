import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read50() {
  invoke03(() => query("/* entry_50 */ SELECT 50"), payload);
  return query("SELECT id FROM items WHERE id = 50"); // expected-finding
}
