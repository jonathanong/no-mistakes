import { mkdtempSync } from "node:fs";
import { main } from "../src/main.mts";
function mutate() { this.root = process.cwd(); }
main({ nested: { root: mkdtempSync("/tmp/test-"), mutate } });
