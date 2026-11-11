import { readdirSync, mkdtempSync } from "node:fs";
const options = { nested: { root: mkdtempSync("/tmp/test-") } };
readdirSync(options.nested.root);
