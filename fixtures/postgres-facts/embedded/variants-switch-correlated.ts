import { query } from "@example/db";
let prefix = "SELECT ";
let suffix = "0";
switch (mode) {
  case "a": prefix = "SELECT a"; suffix = "1";
  case "b": suffix = "2"; break;
  default: prefix = "SELECT z"; suffix = "9";
}
query(prefix + suffix); // SELECT a2, SELECT 2, SELECT z9, and no cross-path pair.
