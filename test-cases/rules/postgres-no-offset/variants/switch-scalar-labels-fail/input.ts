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
