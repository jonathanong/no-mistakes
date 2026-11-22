import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read37() {
  invoke03(() => query("/* entry_37 */ SELECT 37"), payload);
  return query("SELECT id FROM items WHERE id = 37"); // expected-finding
}
