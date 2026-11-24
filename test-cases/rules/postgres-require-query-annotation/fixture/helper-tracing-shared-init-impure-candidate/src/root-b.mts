import { query } from "@bench/db";
import { firstStatement } from "./impure-array.mjs";

export function runB() {
  return query(firstStatement());
}
