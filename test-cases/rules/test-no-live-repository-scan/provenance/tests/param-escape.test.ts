// Reported: the function is also passed by reference, so a caller omits the owned directory.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { syncEscaped } from "../src/param-escape";

const ownedDir = mkdtempSync(join(tmpdir(), "param-escape-"));
syncEscaped("admin", ownedDir);
["user"].forEach(syncEscaped);
