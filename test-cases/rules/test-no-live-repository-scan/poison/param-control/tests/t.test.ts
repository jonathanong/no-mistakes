// Accepted: the only caller passes an owned directory.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list as direct } from "../src/dir";

const ownedDir = mkdtempSync(join(tmpdir(), "escape-"));
direct("a", ownedDir);
