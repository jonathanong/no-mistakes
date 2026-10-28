// Flagged only when `enumerators` adds `cpSync`; copying a tree walks it as well.
import { cpSync } from "node:fs";

cpSync(".", "out", { recursive: true });
