import { readdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
const root = mkdtempSync(join(tmpdir(), "prototype-"));
const project = { __proto__: { root: process.cwd() } };
readdirSync(resolve(root, project.root ?? "."));
