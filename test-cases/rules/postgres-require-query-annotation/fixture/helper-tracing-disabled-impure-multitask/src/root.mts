import { query } from "@bench/db";
import { firstStatement } from "./impure-array.mjs";

export function entry0() {
  return query(firstStatement());
}

export function entry1() {
  return query(firstStatement());
}

export function entry2() {
  return query(firstStatement());
}

export function entry3() {
  return query(firstStatement());
}
