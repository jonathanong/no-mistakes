import { readdirSync, mkdtempSync } from "node:fs";
import { pathToFileURL } from "node:url";
function scan(options) { return readdirSync(options.nested.root); }
function main(options) {
  options = { nested: { root: process.cwd() } };
  return scan(options);
}
main({ nested: { root: mkdtempSync("/tmp/root-") } });
