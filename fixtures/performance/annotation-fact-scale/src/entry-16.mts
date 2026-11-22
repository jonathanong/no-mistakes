import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read16() {
  invoke03(() => query("/* entry_16 */ SELECT 16"), payload);
  return query("SELECT id FROM items WHERE id = 16"); // expected-finding
}
