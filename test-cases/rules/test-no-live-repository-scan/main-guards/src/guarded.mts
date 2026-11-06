import { readdirSync } from "node:fs";
export function scanRepo() {
  return readdirSync(".");
}
export function formatReport() { return "report"; }
if (import.meta.main) {
  scanRepo();
  (() => readdirSync("./inline-callback"))();
  consume(scanRepo);
  readdirSync("./bare-guard");
}
if (import.meta.main === true) {
  readdirSync("./strict-true");
}
if (import.meta.main && formatReport()) {
  readdirSync("./main-first");
}
if (formatReport() && import.meta.main) {
  readdirSync("./main-last");
}
if ((import.meta.main === true) && formatReport()) {
  readdirSync("./strict-conjunct");
}

function consume(callback: () => unknown) { return callback(); }
