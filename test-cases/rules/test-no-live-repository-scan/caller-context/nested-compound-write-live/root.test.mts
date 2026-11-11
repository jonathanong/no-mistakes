import { readdirSync, mkdtempSync } from "node:fs";
const options = { nested: { root: mkdtempSync("/tmp/test-") } };
options.nested.root += "/../repository";
readdirSync(options.nested.root);
