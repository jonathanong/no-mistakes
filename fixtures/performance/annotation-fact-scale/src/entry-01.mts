import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read01() {
  invoke03(() => query("/* entry_01 */ SELECT 1"), payload);
  return query("SELECT id FROM items WHERE id = 1"); // expected-finding
}
