import { readdirSync, mkdtempSync } from "node:fs";
const options = { nested: { root: mkdtempSync("/tmp/test-") } };
options.nested.root = process.cwd();
readdirSync(options.nested.root);
