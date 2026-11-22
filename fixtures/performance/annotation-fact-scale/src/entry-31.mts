import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read31() {
  invoke03(() => query("/* entry_31 */ SELECT 31"), payload);
  return query("SELECT id FROM items WHERE id = 31"); // expected-finding
}
