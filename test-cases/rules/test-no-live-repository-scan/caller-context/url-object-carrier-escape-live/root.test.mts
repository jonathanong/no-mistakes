import { readdirSync, mkdtempSync } from "node:fs";
import { pathToFileURL } from "node:url";
const url = pathToFileURL(mkdtempSync("/tmp/url-"));
const carrier = { root: url };
function mutateCarrier(value) { value.root.pathname = process.cwd(); }
mutateCarrier(carrier);
readdirSync(url);
