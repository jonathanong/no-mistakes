import { readdirSync, mkdtempSync } from "node:fs";
import { pathToFileURL, fileURLToPath } from "node:url";
const options = { nested: { root: pathToFileURL(mkdtempSync("/tmp/url-")) } };
function mutate(value) { if (value instanceof URL) value.pathname = process.cwd(); }
mutate(options.nested.root);
readdirSync(options.nested.root);
