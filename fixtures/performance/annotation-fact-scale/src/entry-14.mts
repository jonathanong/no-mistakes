import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read14() {
  invoke03(() => query("/* entry_14 */ SELECT 14"), payload);
  return query("SELECT id FROM items WHERE id = 14"); // expected-finding
}
