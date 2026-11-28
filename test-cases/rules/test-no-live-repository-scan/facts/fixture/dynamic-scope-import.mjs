import { readFileSync } from "node:fs";

export function load(scope) {
  with (scope) {
    readFileSync("reports/a.txt");
  }
}
