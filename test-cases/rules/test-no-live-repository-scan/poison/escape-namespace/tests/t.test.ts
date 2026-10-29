// Reported: the function is called through a namespace destructure.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list as direct } from "../src/dir";
import * as ns from "../src/dir";

const ownedDir = mkdtempSync(join(tmpdir(), "escape-"));
direct("a", ownedDir);
const { list: viaNamespace } = ns;
viaNamespace("u");
