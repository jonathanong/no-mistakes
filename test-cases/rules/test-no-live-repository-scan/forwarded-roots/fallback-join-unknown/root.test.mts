import { readdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
const root = mkdtempSync(join(tmpdir(), "join-fallback-"));
readdirSync(join(root, project.root ?? project.test?.root ?? "."));
