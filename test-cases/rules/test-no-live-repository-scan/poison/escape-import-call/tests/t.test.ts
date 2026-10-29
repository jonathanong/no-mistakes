// Reported: the function is called through a dynamic import namespace.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list as direct } from "../src/dir";

const ownedDir = mkdtempSync(join(tmpdir(), "escape-"));
direct("a", ownedDir);
import("../src/dir").then((loaded) => loaded.list("u"));
