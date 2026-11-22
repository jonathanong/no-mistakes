import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read23() {
  invoke03(() => query("/* entry_23 */ SELECT 23"), payload);
  return query("SELECT id FROM items WHERE id = 23"); // expected-finding
}
