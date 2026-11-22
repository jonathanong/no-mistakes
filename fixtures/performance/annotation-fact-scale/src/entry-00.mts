import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read00() {
  invoke03(() => query("/* entry_00 */ SELECT 0"), payload);
  return query("SELECT id FROM items WHERE id = 0"); // expected-finding
}
