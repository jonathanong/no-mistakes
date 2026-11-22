import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read19() {
  invoke03(() => query("/* entry_19 */ SELECT 19"), payload);
  return query("SELECT id FROM items WHERE id = 19"); // expected-finding
}
