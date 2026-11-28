import { query } from "@bench/db";
import { label } from "./helper.mjs";

export function entry0() {
  return query([label(), "SELECT 1"].join(" "));
}

export function entry1() {
  return query([label(), "SELECT 1"].join(" "));
}

export function entry2() {
  return query([label(), "SELECT 1"].join(" "));
}

export function entry3() {
  return query([label(), "SELECT 1"].join(" "));
}
