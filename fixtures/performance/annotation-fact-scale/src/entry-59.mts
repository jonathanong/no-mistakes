import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read59() {
  invoke03(() => query("/* entry_59 */ SELECT 59"), payload);
  return query("SELECT id FROM items WHERE id = 59"); // expected-finding
}
