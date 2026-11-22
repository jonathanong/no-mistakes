import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read53() {
  invoke03(() => query("/* entry_53 */ SELECT 53"), payload);
  return query("SELECT id FROM items WHERE id = 53"); // expected-finding
}
