import { query } from "@example/db";
let callGuard = false;
switch (2) {
  case (function () { callGuard = true; return 1; }).call(null): break;
  case 2: query(callGuard ? "SELECT 1 OFFSET 1" : "SELECT 1"); break;
}
let applyGuard = false;
switch (2) {
  case (() => { applyGuard = true; return 1; }).apply(null, []): break;
  case 2: query(applyGuard ? "SELECT 2 OFFSET 1" : "SELECT 2"); break;
}
let middle = false;
switch (3) {
  case 1: break;
  default: query(middle ? "SELECT 3 OFFSET 1" : "SELECT 3"); break;
  case (function () { middle = true; return 2; })["call"](null): break;
}
let argumentGuard = false;
switch (2) {
  case (() => 1).apply(null, [(argumentGuard = true)]): break;
  case 2: query(argumentGuard ? "SELECT 4 OFFSET 1" : "SELECT 4"); break;
}
let literalGuard = false;
switch (2) {
  case (function () { literalGuard = true; return 1; }).call(null): break;
  case 2: query("SELECT 5 OFFSET 1"); break;
}
let suppressed = false;
switch (2) {
  case (() => { suppressed = true; return 1; }).apply(null, []): break;
  case 2:
    // no-mistakes-disable-next-line postgres-no-offset
    query(suppressed ? "SELECT 6 OFFSET 1" : "SELECT 6");
    break;
}
