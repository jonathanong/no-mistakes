import { query } from "@example/db";
const gate = runtimeFlag;
let ifText = gate ? "SELECT 1" : "SELECT 2";
if (gate) ifText += "0";
query(ifText);
let conditionalText = gate ? "SELECT 1" : "SELECT 2";
gate ? conditionalText += "0" : conditionalText += "9";
query(conditionalText);
let andText = gate ? "SELECT 1" : "SELECT 2";
gate && (andText += "0");
query(andText);
let orText = gate ? "SELECT 1" : "SELECT 2";
gate || (orText += "9");
query(orText);
let contradictoryText = gate ? "SELECT 1" : "SELECT 2";
// The inner true arm cannot run on the immutable gate's outer true path.
if (gate) {
  if (!gate) contradictoryText = unknownSql;
  else contradictoryText += "0";
}
query(contradictoryText);
let switchText = gate ? "SELECT 1" : "SELECT 2";
if (gate) {
  switch (runtimeMode) {
    case "a": switchText += "0"; break;
    default: switchText += "9";
  }
}
query(switchText);
let nestedSwitchText = "SELECT ";
switch (runtimeMode) {
  case "a": nestedSwitchText += "1"; // Intentional fallthrough retains both entry paths.
  case "b":
    switch (runtimeSubMode) {
      case "x": nestedSwitchText += "0"; break;
      default: nestedSwitchText += "9";
    }
    break;
  default: nestedSwitchText += "2";
}
query(nestedSwitchText);
