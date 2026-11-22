import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read04() {
  invoke03(() => query("/* entry_04 */ SELECT 4"), payload);
  return query("SELECT id FROM items WHERE id = 4"); // expected-finding
}
