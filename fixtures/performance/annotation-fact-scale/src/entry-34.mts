import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read34() {
  invoke03(() => query("/* entry_34 */ SELECT 34"), payload);
  return query("SELECT id FROM items WHERE id = 34"); // expected-finding
}
