import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read33() {
  invoke03(() => query("/* entry_33 */ SELECT 33"), payload);
  return query("SELECT id FROM items WHERE id = 33"); // expected-finding
}
