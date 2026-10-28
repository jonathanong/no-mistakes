// Negative: reads rooted under a fixtures directory are bounded input.
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";

const fixtures = join(import.meta.dirname, "fixtures", "workflows");
readdirSync(fixtures);
readdirSync(`${import.meta.dirname}/__fixtures__/jobs`);
// A finite literal list is not an enumeration at all.
["a.yml", "b.yml"].map((name) => readFileSync(join(fixtures, name), "utf8"));
