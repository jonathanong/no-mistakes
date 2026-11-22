import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read52() {
  invoke03(() => query("/* entry_52 */ SELECT 52"), payload);
  return query("SELECT id FROM items WHERE id = 52"); // expected-finding
}
