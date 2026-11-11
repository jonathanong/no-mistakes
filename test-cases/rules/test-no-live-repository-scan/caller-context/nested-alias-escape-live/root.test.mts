import { readdirSync, mkdtempSync } from "node:fs";
const options = { nested: { root: mkdtempSync("/tmp/test-") } };
const alias = options.nested;
alias.root = process.cwd();
readdirSync(options.nested.root);
