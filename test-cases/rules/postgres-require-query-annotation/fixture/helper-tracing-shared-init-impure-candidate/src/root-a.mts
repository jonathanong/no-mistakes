import { query } from "@bench/db";
import { firstStatement } from "./impure-array.mjs";

export function runA() {
  return query(firstStatement());
}
