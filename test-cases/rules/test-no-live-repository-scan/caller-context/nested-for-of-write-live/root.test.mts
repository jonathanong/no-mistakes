import { readdirSync, mkdtempSync } from "node:fs";
const options = { nested: { root: mkdtempSync("/tmp/test-") } };
for (options.nested.root of [process.cwd()]) {}
readdirSync(options.nested.root);
