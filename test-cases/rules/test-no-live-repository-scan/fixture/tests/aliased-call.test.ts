// Positive: the same enumeration through renamed, default, and namespace imports.
import fs from "node:fs";
import * as nodeFs from "node:fs";
import { readdir } from "node:fs/promises";
import { readdirSync as ls } from "node:fs";

ls(process.cwd());
fs.readdirSync(".github/workflows");
nodeFs.readdirSync("packages");
await readdir("/srv/repository");
await fs.promises.readdir("docs");
