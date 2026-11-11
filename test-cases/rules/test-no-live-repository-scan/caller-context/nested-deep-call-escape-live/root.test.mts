import { readdirSync, mkdtempSync } from "node:fs";
const options = { outer: { nested: { root: mkdtempSync("/tmp/test-") } } };
function mutate(value) { value.root = process.cwd(); }
mutate(options.outer.nested);
readdirSync(options.outer.nested.root);
