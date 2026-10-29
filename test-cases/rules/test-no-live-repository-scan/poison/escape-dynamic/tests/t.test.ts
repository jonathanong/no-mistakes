// Reported: a function destructured from import() is passed as a value.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list as direct } from "../src/dir";

const ownedDir = mkdtempSync(join(tmpdir(), "escape-"));
direct("a", ownedDir);
import("../src/dir").then(({ list: renamed }) => ["u"].map(renamed));
