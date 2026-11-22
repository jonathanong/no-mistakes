import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read58() {
  invoke03(() => query("/* entry_58 */ SELECT 58"), payload);
  return query("SELECT id FROM items WHERE id = 58"); // expected-finding
}
