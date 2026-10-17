import { query } from "@example/db";

/* Literal function and arrow bodies execute through call and apply. */
let functionCall = false;
switch (2) {
  case (function () { functionCall = true; return 1; }).call(null): break;
  case 2: query(functionCall ? "SELECT 1 OFFSET 1" : "SELECT 1"); break; // opaque
}
let functionApply = false;
switch (2) {
  case (function () { functionApply = true; return 1; }).apply(null, []): break;
  case 2: query(functionApply ? "SELECT 2 OFFSET 1" : "SELECT 2"); break; // opaque
}
let arrowCall = false;
switch (2) {
  case (() => { arrowCall = true; return 1; }).call(null): break;
  case 2: query(arrowCall ? "SELECT 3 OFFSET 1" : "SELECT 3"); break; // opaque
}
let arrowApply = false;
switch (2) {
  case (() => { arrowApply = true; return 1; }).apply(null, []): break;
  case 2: query(arrowApply ? "SELECT 4 OFFSET 1" : "SELECT 4"); break; // opaque
}

/* Wrappers around the callee, receiver, and literal property preserve invocation. */
let wrappedFunction = false;
switch (2) {
  case (((function () { wrappedFunction = true; return 1; }) as () => number).call as any)(null): break;
  case 2: query(wrappedFunction ? "SELECT 5 OFFSET 1" : "SELECT 5"); break; // opaque
}
let wrappedArrow = false;
switch (2) {
  case (((() => { wrappedArrow = true; return 1; }) satisfies () => number)!).apply(null, []): break;
  case 2: query(wrappedArrow ? "SELECT 6 OFFSET 1" : "SELECT 6"); break; // opaque
}
let computedCall = false;
switch (2) {
  case (function () { computedCall = true; return 1; })["call"](null): break;
  case 2: query(computedCall ? "SELECT 7 OFFSET 1" : "SELECT 7"); break; // opaque
}
let computedApply = false;
switch (2) {
  case (() => { computedApply = true; return 1; })["apply" as const](null, []): break;
  case 2: query(computedApply ? "SELECT 8 OFFSET 1" : "SELECT 8"); break; // opaque
}

/* Callee and argument writes occur before the invoked body. */
let callArgument = false;
switch (2) {
  case (function () { return 1; }).call((callArgument = true, null)): break;
  case 2: query(callArgument ? "SELECT 9 OFFSET 1" : "SELECT 9"); break; // opaque
}
let applyArgument = false;
switch (2) {
  case (() => 1).apply(null, [(applyArgument = true)]): break;
  case 2: query(applyArgument ? "SELECT 10 OFFSET 1" : "SELECT 10"); break; // opaque
}
let propertyWrite = false;
switch (2) {
  case (() => 1)[(propertyWrite = true, "call")](null): break;
  case 2: query(propertyWrite ? "SELECT 11 OFFSET 1" : "SELECT 11"); break; // opaque
}
let nestedCall = false;
switch (2) {
  case (() => { (function () { ({ nestedCall } = { nestedCall: true }); }).call(null); return 1; }).apply(null, []): break;
  case 2: query(nestedCall ? "SELECT 12 OFFSET 1" : "SELECT 12"); break; // opaque
}

/* Default selection evaluates later labels, while fallthrough skips them. */
let middle = false;
switch (3) {
  case 1: break;
  default: query(middle ? "SELECT 13 OFFSET 1" : "SELECT 13"); break; // opaque
  case (function () { middle = true; return 2; }).call(null): break;
}
let fallthrough = false;
switch (1) {
  case 1:
  case (() => { fallthrough = true; return 2; }).apply(null, []):
    query(fallthrough ? "SELECT 14 OFFSET 1" : "SELECT 14"); // opaque
    break;
}

/* Creating a member reference or passing a callback does not invoke its body. */
let declared = false;
switch (2) {
  case (function () { declared = true; }).call: break;
  case 2: query(declared ? "SELECT 15 OFFSET 1" : "SELECT 15"); break; // finite: SELECT 15
}
let passed = false;
switch (2) {
  case acceptCallback((function () { passed = true; }).apply): break;
  case 2: query(passed ? "SELECT 16 OFFSET 1" : "SELECT 16"); break; // finite: SELECT 16
}
let argumentCallback = false;
switch (2) {
  case (function () { return 1; }).call(null, () => { argumentCallback = true; }): break;
  case 2: query(argumentCallback ? "SELECT 17 OFFSET 1" : "SELECT 17"); break; // finite: SELECT 17
}
let nestedCallback = false;
switch (2) {
  case (() => { const callback = function () { nestedCallback = true; }; return 1; }).apply(null, []): break;
  case 2: query(nestedCallback ? "SELECT 18 OFFSET 1" : "SELECT 18"); break; // finite: SELECT 18
}
let bound = false;
switch (2) {
  case (function () { bound = true; }).bind(null): break;
  case 2: query(bound ? "SELECT 19 OFFSET 1" : "SELECT 19"); break; // finite: SELECT 19
}
let computedBound = false;
switch (2) {
  case (() => { computedBound = true; })["bind"](null): break;
  case 2: query(computedBound ? "SELECT 20 OFFSET 1" : "SELECT 20"); break; // finite: SELECT 20
}
let ordinary = false;
switch (2) {
  case Math[property](1): break;
  case 2: query(ordinary ? "SELECT 21 OFFSET 1" : "SELECT 21"); break; // finite: SELECT 21
}

/* Pure invoked bodies and legacy literals retain their ordinary facts. */
let pure = false;
switch (2) {
  case (() => 1).call(null): break;
  case 2: query(pure ? "SELECT 22 OFFSET 1" : "SELECT 22"); break; // finite: SELECT 22
}
const plain = "SELECT 23";
let literal = false;
switch (2) {
  case (function () { literal = true; return 1; }).apply(null, []): break;
  case 2:
    query(plain); // unchanged: SELECT 23
    query("SELECT 24 OFFSET 1"); // inline: SELECT 24 OFFSET 1
    break;
}
