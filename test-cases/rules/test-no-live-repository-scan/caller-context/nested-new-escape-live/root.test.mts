import { readdirSync, mkdtempSync } from "node:fs";
const options = { nested: { root: mkdtempSync("/tmp/test-") } };
class Mutator { constructor(value) { value.root = process.cwd(); } }
new Mutator(options.nested);
readdirSync(options.nested.root);
