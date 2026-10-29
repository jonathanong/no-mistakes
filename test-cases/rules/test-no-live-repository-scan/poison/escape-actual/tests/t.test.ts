// Reported: the function is called through vi.importActual.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list as direct } from "../src/dir";
import { vi } from "vitest";

const ownedDir = mkdtempSync(join(tmpdir(), "escape-"));
direct("a", ownedDir);
vi.importActual<typeof import("../src/dir")>("../src/dir").then((actual) => actual.list("u"));
