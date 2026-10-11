import { query, sql } from "@example/db";
let q = "SELECT 1";
switch (mode) {
  case "a": q += " /* a */"; break;
  case "b": q += " /* b */"; break;
  default: q += " /* c */";
}
query(q);
let fall = "SELECT 2";
switch (mode) {
  case "a": fall += " /* a */";
  case "b": fall += " /* b */"; break;
}
query(fall);
