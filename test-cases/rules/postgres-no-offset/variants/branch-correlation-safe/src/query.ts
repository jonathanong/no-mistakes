import { query } from "@example/db";

// OFFSET exists only on the path that each guard excludes.
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

function logical(gate: boolean) {
  let flag;
  let statement;
  if (gate) { flag = true; statement = "SELECT 7"; }
  else { flag = false; statement = "SELECT 8 OFFSET 1"; }
  flag && query(statement);
  flag ? query(statement) : consumeElsewhere();
}
