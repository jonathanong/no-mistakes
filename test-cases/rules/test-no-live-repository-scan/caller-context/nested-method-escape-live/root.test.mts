import { readdirSync, mkdtempSync } from "node:fs";
function mutate() { this.root = process.cwd(); }
const options = { nested: { root: mkdtempSync("/tmp/test-"), mutate } };
options.nested.mutate();
readdirSync(options.nested.root);
