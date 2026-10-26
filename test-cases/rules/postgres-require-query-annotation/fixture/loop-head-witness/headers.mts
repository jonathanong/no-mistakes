import { read } from "@bench/db";

function infinite() { let q = "/* named */ SELECT 1"; for (;;) { read(q); q = "SELECT 2"; } } // witness
function booleanTest() { let q = "/* named */ SELECT 1"; while (true) { read(q); q = "SELECT 2"; } } // witness
function negated(done) { let q = "/* named */ SELECT 1"; while (!done) { read(q); q = "SELECT 2"; } } // witness
function logical(a, b) { let q = "/* named */ SELECT 1"; while (a && b) { read(q); q = "SELECT 2"; } } // witness
function nestedBinary(i, count) { let q = "/* named */ SELECT 1"; while (i + 1 < count) { read(q); q = "SELECT 2"; } } // witness
function booleanOperand(i) { let q = "/* named */ SELECT 1"; while (i !== true) { read(q); q = "SELECT 2"; } } // witness
function stringOperand(i) { let q = "/* named */ SELECT 1"; while (i !== "done") { read(q); q = "SELECT 2"; } } // witness
function nullOperand(i) { let q = "/* named */ SELECT 1"; while (i !== null) { read(q); q = "SELECT 2"; } } // witness
function pureUpdate(i) { let q = "/* named */ SELECT 1"; for (; i; i + 1) { read(q); q = "SELECT 2"; } } // witness
function harmlessBranch(xs, flag) { let q = "/* named */ SELECT 1"; for (const x of xs) { ; debugger; const label = x; if (flag) { console.log(label); } else console.log(x); read(q); q = "SELECT 2"; } } // witness
function callsTwice(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); read(q); q = "SELECT 2"; } } // two witnesses
// Unsupported headers and bodies without a surviving backedge stay silent.
function falseTest() { let q = "/* named */ SELECT 1"; while (false) { read(q); q = "SELECT 2"; } }
function falseDo() { let q = "/* named */ SELECT 1"; do { read(q); q = "SELECT 2"; } while (false); }
function literalTest() { let q = "/* named */ SELECT 1"; while ("yes") { read(q); q = "SELECT 2"; } }
function constantBinary() { let q = "/* named */ SELECT 1"; while (0 > 1) { read(q); q = "SELECT 2"; } }
function effectfulTest(again) { let q = "/* named */ SELECT 1"; while (again()) { read(q); q = "SELECT 2"; } }
function effectfulBinary(again, i) { let q = "/* named */ SELECT 1"; while (again() < i) { read(q); q = "SELECT 2"; } }
function effectfulRight(again, i) { let q = "/* named */ SELECT 1"; while (i < again()) { read(q); q = "SELECT 2"; } }
function otherUnary(i) { let q = "/* named */ SELECT 1"; while (typeof i) { read(q); q = "SELECT 2"; } }
function constantUnary() { let q = "/* named */ SELECT 1"; while (!true) { read(q); q = "SELECT 2"; } }
function badUpdate(i) { let q = "/* named */ SELECT 1"; for (; i; again()) { read(q); q = "SELECT 2"; } }
function propertyUpdate(i) { let q = "/* named */ SELECT 1"; for (; i; object.i++) { read(q); q = "SELECT 2"; } }
function assignmentBeforeRead(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { q = "SELECT 2"; read(q); } }
function branchReturn(xs, flag) { let q = "/* named */ SELECT 1"; for (const x of xs) { if (flag) console.log(x); else return; read(q); q = "SELECT 2"; } } // witness
function conditionalAssignment(xs, flag) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); if (flag) q = "SELECT 2"; } }
function continueBackedge(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); q = "SELECT 2"; continue; q = "/* dead */ SELECT 3"; } } // witness
function initializer(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { const rows = read(q); q = "SELECT 2"; console.log(rows); } } // witness
function continueBeforeAssignment(xs) { let q = "/* named */ SELECT 1"; for (const x of xs) { read(q); continue; q = "SELECT 2"; } }
function labelledContinue(xs) { let q = "/* named */ SELECT 1"; outer: for (const x of xs) { read(q); q = "SELECT 2"; continue outer; } }
