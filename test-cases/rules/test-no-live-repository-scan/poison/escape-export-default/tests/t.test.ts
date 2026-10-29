// Reported: `export default list` is also passed under another local name.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import run from "../src/dir";
import go from "../src/dir";

const ownedDir = mkdtempSync(join(tmpdir(), "escape-"));
run("a", ownedDir);
["u"].map(go);
