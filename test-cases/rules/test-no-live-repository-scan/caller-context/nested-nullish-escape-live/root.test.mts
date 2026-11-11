import { readdirSync, mkdtempSync } from "node:fs";
import { resolve } from "node:path";
const root = mkdtempSync("/tmp/test-");
const options = { nested: { root: null } };
function mutate(value) { value.root = process.cwd(); }
mutate(options.nested);
readdirSync(resolve(root, options.nested.root ?? "."));
