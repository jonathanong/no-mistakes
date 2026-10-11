import { query } from "@example/db";
let chosen = "SELECT ";
switch ("a") {
  case "a": chosen += "1"; break;
  case "b": chosen += "2"; break;
  default: chosen += "3";
}
query(chosen);
let middle = "SELECT ";
switch ("missing") {
  case "a": middle += "a"; break;
  default: middle += "default";
  case "b": middle += "b";
}
query(middle); // A default in the middle falls through to the following case.
let numeric = "SELECT ";
switch (1) {
  case "1": numeric += "text"; break;
  case 1: numeric += "number"; break;
  default: numeric += "other";
}
query(numeric); // Case selection uses JavaScript strict equality.
let boolean = "SELECT ";
switch (true) {
  case false: boolean += "false"; break;
  case true: boolean += "true"; break;
}
query(boolean);
let absent = "SELECT 4";
switch (null) {
  case "null": absent += " OFFSET 1"; break;
}
query(absent); // No match keeps only the unchanged path.
