import { readdirSync, mkdtempSync } from "node:fs";
const options = { nested: { root: mkdtempSync("/tmp/test-") } };
function mutate(value) { value.root = process.cwd(); }
mutate(options.nested);
readdirSync(options.nested.root);
