import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read56() {
  invoke03(() => query("/* entry_56 */ SELECT 56"), payload);
  return query("SELECT id FROM items WHERE id = 56"); // expected-finding
}
