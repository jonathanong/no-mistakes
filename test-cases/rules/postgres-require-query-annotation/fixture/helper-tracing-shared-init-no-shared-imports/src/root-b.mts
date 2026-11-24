import { query } from "@bench/db";

export function runB() {
  return query("/* ok_b */ SELECT 2");
}
