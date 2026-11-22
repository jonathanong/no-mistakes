import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read32() {
  invoke03(() => query("/* entry_32 */ SELECT 32"), payload);
  return query("SELECT id FROM items WHERE id = 32"); // expected-finding
}
