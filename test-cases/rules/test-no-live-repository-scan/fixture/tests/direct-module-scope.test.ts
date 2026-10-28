// Positive: the downstream workflow-policy shape. A module-scope enumeration
// of live repository files followed by a read of each one.
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const workflows = join(import.meta.dirname, "..", ".github", "workflows");
const entries = readdirSync(workflows).map((name) => readFileSync(join(workflows, name), "utf8"));

export { entries };
