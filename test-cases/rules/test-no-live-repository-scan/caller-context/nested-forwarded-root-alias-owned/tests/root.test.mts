import { mkdtempSync } from "node:fs";
import { main } from "../src/main.mts";
main({ nested: { root: mkdtempSync("/tmp/test-") } });
