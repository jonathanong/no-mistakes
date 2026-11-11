import { readdirSync, mkdtempSync, mkdtempDisposableSync } from "node:fs";
import { pathToFileURL } from "node:url";
const root = pathToFileURL(mkdtempSync("/tmp/url-"));
function mutate(url) { url.pathname = process.cwd(); }
mutate(root);
readdirSync(root);
