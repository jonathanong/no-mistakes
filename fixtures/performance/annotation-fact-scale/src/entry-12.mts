import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read12() {
  invoke03(() => query("/* entry_12 */ SELECT 12"), payload);
  return query("SELECT id FROM items WHERE id = 12"); // expected-finding
}
