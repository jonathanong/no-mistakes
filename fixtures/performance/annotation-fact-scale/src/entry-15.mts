import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read15() {
  invoke03(() => query("/* entry_15 */ SELECT 15"), payload);
  return query("SELECT id FROM items WHERE id = 15"); // expected-finding
}
