// Reported: the callee is consumed through an alias whose spelling differs from the file name.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list as direct } from "../src/dir";
import * as ns from "@lib";

const ownedDir = mkdtempSync(join(tmpdir(), "escape-"));
direct("a", ownedDir);
const { list: viaNamespace } = ns;
viaNamespace("u");
