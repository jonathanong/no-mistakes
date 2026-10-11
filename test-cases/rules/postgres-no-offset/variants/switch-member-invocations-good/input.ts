import { query } from "@example/db";
let pure = false;
switch (2) {
  case (() => 1).call(null): break;
  case 2: query(pure ? "SELECT 1 OFFSET 1" : "SELECT 1"); break;
}
let callbackGuard = false;
switch (2) {
  case (function () { return 1; }).apply(null, [() => { callbackGuard = true; }]): break;
  case 2: query(callbackGuard ? "SELECT 2 OFFSET 1" : "SELECT 2"); break;
}
let declared = false;
switch (2) {
  case acceptCallback((function () { declared = true; }).call): break;
  case 2: query(declared ? "SELECT 3 OFFSET 1" : "SELECT 3"); break;
}
const statement = "SELECT 4";
let literalGuard = false;
switch (2) {
  case (function () { literalGuard = true; return 1; }).call(null): break;
  case 2:
    query("SELECT 5");
    query(statement);
    break;
}
