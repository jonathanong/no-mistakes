import { readdirSync, mkdtempSync, mkdtempDisposableSync } from "node:fs";
import { pathToFileURL } from "node:url";
const root = pathToFileURL(mkdtempSync("/tmp/url-"));
const alias = root;
alias.pathname = process.cwd();
readdirSync(root);
