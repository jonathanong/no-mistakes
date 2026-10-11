import { query } from "@example/db";

function nestedFunction() {
  var statement = "SELECT 1";
  function inner() {
    if (true) { var statement = "/* inner */ SELECT 2"; }
    query(statement);
  }
  inner();
  query(statement);
}

function trueGuard() {
  var statement = "SELECT 3";
  if (true) { const unrelated = 1; }
  query(statement);
}

function falseGuard() {
  var statement = "SELECT 4";
  if (false) { const unrelated = 1; }
  query(statement);
}

function nestedGuards() {
  var statement = "SELECT 5";
  if (false) { if (true) { const unrelated = 1; } }
  query(statement);
}

function conditionalGuard() {
  var statement = "SELECT 6";
  true ? consumeElsewhere() : consumeElsewhere();
  query(statement);
}

function logicalGuard() {
  var statement = "SELECT 7";
  true && consumeElsewhere();
  false || consumeElsewhere();
  null ?? consumeElsewhere();
  query(statement);
}

// Visiting a skipped mutation may clear legacy SQL; its safe version survives.
function skippedMutation() {
  let statement = "SELECT 8";
  if (false) statement += " OFFSET 1";
  query(statement);
}

function skippedNullishMutation() {
  let statement = "SELECT 9";
  0 ?? (statement += " OFFSET 1");
  query(statement);
}
