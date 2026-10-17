import { query } from "@example/db";

function positive(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 1"; }
  else { flag = false; statement = "SELECT 2 OFFSET 1"; }
  if (flag) query(statement);
}

function alternate(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 3 OFFSET 1"; }
  else { flag = false; statement = "SELECT 4"; }
  if (flag) consumeElsewhere(); else query(statement);
}

function inverted(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 5 OFFSET 1"; }
  else { flag = false; statement = "SELECT 6"; }
  if (!flag) query(statement);
}

function nested(gate: boolean, outer: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 7"; }
  else { flag = false; statement = "SELECT 8 OFFSET 1"; }
  if (outer) { if (!!flag) query(statement); }
}

function logicalAnd(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 9"; }
  else { flag = false; statement = "SELECT 10 OFFSET 1"; }
  flag && query(statement);
}

function logicalOr(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 11 OFFSET 1"; }
  else { flag = false; statement = "SELECT 12"; }
  flag || query(statement);
}

function nullish(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 13 OFFSET 1"; }
  else { flag = null; statement = "SELECT 14"; }
  flag ?? query(statement);
}

function conditional(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 15"; }
  else { flag = false; statement = "SELECT 16 OFFSET 1"; }
  flag ? query(statement) : consumeElsewhere();
}

// Reachable OFFSET branches must remain visible after the path restriction.
function reachable(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 17 OFFSET 1"; }
  else { flag = false; statement = "SELECT 18"; }
  if (flag) query(statement);
  if (flag) consumeElsewhere(); else query(statement);
  if (!flag) query(statement);
}

// Reassigning the guard breaks the old relationship rather than hiding SQL.
function reassigned(gate: boolean, other: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 19"; }
  else { flag = false; statement = "SELECT 20 OFFSET 1"; }
  flag = other;
  if (flag) query(statement);
}
