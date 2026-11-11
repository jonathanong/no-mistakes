import { readdirSync, mkdtempSync, mkdtempDisposableSync } from "node:fs";
import { pathToFileURL } from "node:url";
function scan(...roots) { return readdirSync(roots[0]); }
const roots = JSON.parse(process.argv[2]);
scan(...roots);
