import { readdirSync, mkdtempSync } from "node:fs";
import { pathToFileURL, fileURLToPath } from "node:url";
const url = pathToFileURL(mkdtempSync("/tmp/url-"));
const options = { nested: { root: url } };
readdirSync(options.nested.root);
