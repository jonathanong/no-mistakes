import { query } from "@example/db";

// Reachable OFFSET versions still report at their physical SQL token.
function positive(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 1 OFFSET 1"; } // findings: offset
  else { flag = false; statement = "SELECT 2"; }
  if (flag) query(statement);
}

function alternate(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 3"; }
  else { flag = false; statement = "SELECT 4 OFFSET 1"; } // findings: offset
  if (flag) consumeElsewhere(); else query(statement);
}

function inverted(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 5"; }
  else { flag = false; statement = "SELECT 6 OFFSET 1"; } // findings: offset
  if (!flag) query(statement);
}

function reassigned(gate: boolean, other: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 7"; }
  else { flag = false; statement = "SELECT 8 OFFSET 1"; } // findings: offset
  flag = other;
  if (flag) query(statement);
}
