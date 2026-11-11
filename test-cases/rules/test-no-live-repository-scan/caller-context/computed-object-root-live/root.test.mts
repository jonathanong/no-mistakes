import { readdirSync, mkdtempSync, mkdtempDisposableSync } from "node:fs";
import { pathToFileURL } from "node:url";
function scan(options) { return readdirSync(options.root); }
const key = process.argv[2];
scan({ root: mkdtempSync("/tmp/root-"), [key]: process.cwd() });
