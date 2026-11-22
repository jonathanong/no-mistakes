import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read13() {
  invoke03(() => query("/* entry_13 */ SELECT 13"), payload);
  return query("SELECT id FROM items WHERE id = 13"); // expected-finding
}
