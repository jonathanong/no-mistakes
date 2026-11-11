import { readdirSync, mkdtempSync } from "node:fs";
const options = { nested: { root: mkdtempSync("/tmp/test-") } };
Object.assign(options.nested, { root: process.cwd() });
readdirSync(options.nested.root);
