import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read36() {
  invoke03(() => query("/* entry_36 */ SELECT 36"), payload);
  return query("SELECT id FROM items WHERE id = 36"); // expected-finding
}
