import { query } from "@bench/db";
import { invoke03 } from "./shared/helper-03.mjs";
import { payload } from "./shared/payload.mjs";

export function read63() {
  invoke03(() => query("/* entry_63 */ SELECT 63"), payload);
  return query("SELECT id FROM items WHERE id = 63"); // expected-finding
}
