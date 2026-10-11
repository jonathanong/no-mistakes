import { query } from "@example/db";

/* A failed earlier label changes the guard before the matching case runs. */
let assigned = false;
switch (2) {
  case (assigned = true, 1): break;
  case 2: query(assigned ? "SELECT 1 OFFSET 1" : "SELECT 1"); break; // opaque
}
let updated = false;
switch (2) {
  case (updated++, 1): break;
  case 2: query(updated ? "SELECT 2 OFFSET 1" : "SELECT 2"); break; // opaque
}
let objectGuard = false;
switch (2) {
  case (({ objectGuard } = { objectGuard: true }), 1): break;
  case 2: query(objectGuard ? "SELECT 3 OFFSET 1" : "SELECT 3"); break; // opaque
}
let arrayGuard = false;
switch (2) {
  case ([arrayGuard] = [true], 1): break;
  case 2: query(arrayGuard ? "SELECT 4 OFFSET 1" : "SELECT 4"); break; // opaque
}

/* A default in the middle runs only after every later label fails. */
let middle = false;
switch (3) {
  case 1: break;
  default: query(middle ? "SELECT 5 OFFSET 1" : "SELECT 5"); break; // opaque
  case (middle = true, 2): break;
}

/* Fallthrough skips the later label; recovery must not replay its write. */
let skipped = false;
switch (1) {
  case 1:
  case (skipped = true, 2):
    query(skipped ? "SELECT 6 OFFSET 1" : "SELECT 6"); // opaque
    break;
}

/* Invoked closures execute, but nested callbacks declared in them do not. */
let arrowGuard = false;
switch (2) {
  case (() => { arrowGuard = true; return 1; })(): break;
  case 2: query(arrowGuard ? "SELECT 7 OFFSET 1" : "SELECT 7"); break; // opaque
}
let functionGuard = false;
switch (2) {
  case (function () { functionGuard = true; return 1; })(): break;
  case 2: query(functionGuard ? "SELECT 8 OFFSET 1" : "SELECT 8"); break; // opaque
}
let nestedGuard = false;
switch (2) {
  case (() => { const unused = () => { nestedGuard = true; }; return 1; })(): break;
  case 2: query(nestedGuard ? "SELECT 9 OFFSET 1" : "SELECT 9"); break; // variants: SELECT 9
}
let declaredArrow = false;
switch (2) {
  case (() => { declaredArrow = true; }): break;
  case 2: query(declaredArrow ? "SELECT 10 OFFSET 1" : "SELECT 10"); break; // variants: SELECT 10
}
let declaredFunction = false;
switch (2) {
  case (function () { declaredFunction = true; }): break;
  case 2: query(declaredFunction ? "SELECT 11 OFFSET 1" : "SELECT 11"); break; // variants: SELECT 11
}
let passed = false;
switch (2) {
  case acceptCallback(() => { passed = true; }): break;
  case 2: query(passed ? "SELECT 12 OFFSET 1" : "SELECT 12"); break; // variants: SELECT 12
}

/* Pure calls and writes in the discriminant retain ordinary recovery. */
let pure = false;
switch (2) {
  case (function () { return 1; })(): break;
  case 2: query(pure ? "SELECT 13 OFFSET 1" : "SELECT 13"); break; // variants: SELECT 13
}
let discriminant = false;
switch ((discriminant = true, 2)) {
  case 2: query(discriminant ? "SELECT 14 OFFSET 1" : "SELECT 14"); break; // variants: SELECT 14 OFFSET 1
}

/* The conservative boundary still checks literal SQL and legacy constants. */
const plain = "SELECT 15";
let literalGuard = false;
switch (2) {
  case (literalGuard = true, 1): break;
  case 2:
    query("SELECT 16 OFFSET 1"); // inline: SELECT 16 OFFSET 1
    query(plain); // unchanged: SELECT 15
    break;
}
