import { readdirSync, mkdtempSync } from "node:fs";
import { pathToFileURL } from "node:url";
let options = { nested: { root: mkdtempSync("/tmp/root-") } };
options = { nested: { root: process.cwd() } };
readdirSync(options.nested.root);
