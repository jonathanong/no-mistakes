import { readdirSync, mkdtempSync, mkdtempDisposableSync } from "node:fs";
import { pathToFileURL } from "node:url";
const workspace = mkdtempDisposableSync("/tmp/buffer-", { encoding: "buffer" });
function mutate(buffer) { buffer.write(process.cwd().padEnd(buffer.length, "/")); }
mutate(workspace.path);
readdirSync(workspace.path);
