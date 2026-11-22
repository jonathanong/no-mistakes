import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read46() {
  invoke03(() => query("/* entry_46 */ SELECT 46"), payload);
  return query("SELECT id FROM items WHERE id = 46"); // expected-finding
}
