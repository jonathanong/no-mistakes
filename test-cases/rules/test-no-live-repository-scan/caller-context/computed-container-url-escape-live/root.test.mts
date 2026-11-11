import { readdirSync, mkdtempSync, mkdtempDisposableSync } from "node:fs";
import { pathToFileURL } from "node:url";
const options = { nested: { root: pathToFileURL(mkdtempSync("/tmp/url-")) } };
function mutate(url) { url.pathname = process.cwd(); }
mutate(options["nested"].root);
readdirSync(options.nested.root);
