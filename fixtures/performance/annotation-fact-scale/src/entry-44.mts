import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read44() {
  invoke03(() => query("/* entry_44 */ SELECT 44"), payload);
  return query("SELECT id FROM items WHERE id = 44"); // expected-finding
}
