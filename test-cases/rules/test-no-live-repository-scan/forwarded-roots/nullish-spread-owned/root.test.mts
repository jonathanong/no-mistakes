import { readdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
const root = mkdtempSync(join(tmpdir(), "bounded-"));
const project = { ...{ root: null } };
readdirSync(project.root ?? root);
