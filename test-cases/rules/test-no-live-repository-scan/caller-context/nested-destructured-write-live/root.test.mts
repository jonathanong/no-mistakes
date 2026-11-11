import { readdirSync, mkdtempSync } from "node:fs";
const options = { nested: { root: mkdtempSync("/tmp/test-") } };
({ root: options.nested.root } = { root: process.cwd() });
readdirSync(options.nested.root);
