import { readdirSync } from "node:fs";
if (import.meta.main) {
  readdirSync("./entry-only");
} else {
  readdirSync("./else-on-import");
}
readdirSync("./after-guard");
if (!import.meta.main) {
  readdirSync("./negated-on-import");
}
if (import.meta.main || Math.random()) {
  readdirSync("./disjunction-on-import");
}
