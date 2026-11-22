import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read06() {
  invoke03(() => query("/* entry_06 */ SELECT 6"), payload);
  return query("SELECT id FROM items WHERE id = 6"); // expected-finding
}
