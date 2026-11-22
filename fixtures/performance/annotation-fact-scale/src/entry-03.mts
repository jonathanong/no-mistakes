import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read03() {
  invoke03(() => query("/* entry_03 */ SELECT 3"), payload);
  return query("SELECT id FROM items WHERE id = 3"); // expected-finding
}
