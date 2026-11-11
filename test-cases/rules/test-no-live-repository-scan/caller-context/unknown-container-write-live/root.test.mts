import { readdirSync, mkdtempSync, mkdtempDisposableSync } from "node:fs";
import { pathToFileURL } from "node:url";
const options = { nested: { root: mkdtempSync("/tmp/write-") } };
const key = process.argv[2];
options[key] = { root: process.cwd() };
readdirSync(options.nested.root);
