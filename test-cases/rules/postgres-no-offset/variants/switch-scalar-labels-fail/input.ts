import { query } from "@example/db";
let gate = false;
switch (2) {
  case (gate = true, 1): break;
  case 2: query(gate ? "SELECT 1 OFFSET 1" : "SELECT 1"); break;
}
let middle = false;
switch (3) {
  case 1: break;
  default: query(middle ? "SELECT 2 OFFSET 1" : "SELECT 2"); break;
  case (middle = true, 2): break;
}
let literalGuard = false;
switch (2) {
  case (literalGuard = true, 1): break;
  case 2: query("SELECT 3 OFFSET 1"); break;
}
let suppressed = false;
switch (2) {
  case (suppressed = true, 1): break;
  case 2:
    // no-mistakes-disable-next-line postgres-no-offset
    query(suppressed ? "SELECT 4 OFFSET 1" : "SELECT 4");
    break;
}

/* A failed body cannot erase the original OFFSET-bearing statement. */
let replaced = "SELECT 5 OFFSET 1";
let replacedGuard = false;
switch (2) {
  case (replacedGuard = true, 1): replaced = "SELECT 5";
  case 2: query(replaced); break;
}

/* Possible fallthrough and different DML kinds both stay uncertain. */
let fallthrough = "SELECT 6";
let fallthroughGuard = false;
switch (choice) {
  case (fallthroughGuard = true, 1): fallthrough = "SELECT 6 OFFSET 1";
  case 2: query(fallthrough); break;
}
let dml = "DELETE FROM users";
let dmlGuard = false;
switch (choice) {
  case (dmlGuard = true, 1): dml = "SELECT 7 OFFSET 1";
  case 2: query(dml); break;
}

/* Fresh values and literal SQL still receive ordinary checks. */
let freshGuard = false;
switch (2) {
  case (freshGuard = true, 1): break;
  case 2:
    const fresh = "SELECT 8 OFFSET 1";
    query(fresh);
    query("SELECT 9 OFFSET 1");
    break;
}
