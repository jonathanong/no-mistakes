import { mkdtempSync } from "node:fs";

function budgetHelper(budgetCallback: (root: string) => void) {
  const budgetRoot = mkdtempSync("callback-");
  budgetCallback(budgetRoot);
}

function writeHelper(writeCallback: (root: string) => void) {
  let writtenRoot = mkdtempSync("callback-");
  writtenRoot = "other";
  writeCallback(writtenRoot);
}

function closureHelper(closureCallback: (root: string) => void) {
  const capturedRoot = mkdtempSync("callback-");
  const later = () => closureCallback(capturedRoot);
}
