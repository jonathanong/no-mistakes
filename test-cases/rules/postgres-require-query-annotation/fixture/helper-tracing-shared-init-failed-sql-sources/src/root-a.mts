import { query } from "@bench/db";

export function runA() {
  return query("/* ok_a */ SELECT 1");
}
