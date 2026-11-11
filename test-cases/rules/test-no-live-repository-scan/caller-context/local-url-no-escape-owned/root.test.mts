import { readdirSync, mkdtempSync } from "node:fs";
import { pathToFileURL, fileURLToPath } from "node:url";
const options = { nested: { root: pathToFileURL(mkdtempSync("/tmp/url-")) } };
readdirSync(options.nested.root);
