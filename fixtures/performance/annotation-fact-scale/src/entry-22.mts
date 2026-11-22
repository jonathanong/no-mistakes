import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read22() {
  invoke03(() => query("/* entry_22 */ SELECT 22"), payload);
  return query("SELECT id FROM items WHERE id = 22"); // expected-finding
}
