// Accepted: the only call passes an owned directory for the defaulted parameter.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { sync } from "../src/param-sync";

const ownedDir = mkdtempSync(join(tmpdir(), "param-owned-"));
sync("admin", ownedDir);
