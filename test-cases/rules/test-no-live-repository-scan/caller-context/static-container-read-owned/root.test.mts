import { readdirSync, mkdtempSync, mkdtempDisposableSync } from "node:fs";
import { pathToFileURL } from "node:url";
const options = { nested: { root: mkdtempSync("/tmp/read-") } };
void options.nested.root;
readdirSync(options.nested.root);
