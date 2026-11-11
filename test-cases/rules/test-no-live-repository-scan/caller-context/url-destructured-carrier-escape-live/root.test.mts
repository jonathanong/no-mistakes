import { readdirSync, mkdtempSync } from "node:fs";
import { pathToFileURL } from "node:url";
const url = pathToFileURL(mkdtempSync("/tmp/url-"));
const { root } = { root: url };
function mutate(value) { value.pathname = process.cwd(); }
mutate(root);
readdirSync(url);
