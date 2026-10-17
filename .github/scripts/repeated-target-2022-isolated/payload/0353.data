import { query } from "@example/db";
let q = "SELECT 1";
switch (mode) {
  case "a": q += " /* a */";
  case "b":
    query(q); // A direct match and a fallthrough reach the same call.
    q += " /* b */";
    query(q);
    break;
  default: query(q);
}
let text;
if (flag) { text = "SELECT 2"; query(text); }
else { text = "SELECT 3"; query(text); }
