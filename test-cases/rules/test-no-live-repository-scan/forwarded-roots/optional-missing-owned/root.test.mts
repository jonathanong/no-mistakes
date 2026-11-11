import { readdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
const root = mkdtempSync(join(tmpdir(), "optional-"));
const project = {};
readdirSync(resolve(root, project.test?.root ?? "."));
