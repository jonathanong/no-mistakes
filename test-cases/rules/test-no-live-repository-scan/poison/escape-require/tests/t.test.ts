// Reported: a function destructured from require is passed as a value.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list as direct } from "../src/dir";
declare const require: (id: string) => any;

const ownedDir = mkdtempSync(join(tmpdir(), "escape-"));
direct("a", ownedDir);
const { list } = require("../src/dir");
["u"].map(list);
