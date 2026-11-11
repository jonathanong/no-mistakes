import { mkdtempSync } from "node:fs";
import { main } from "../src/main.mts";
const root = mkdtempSync("/tmp/boundary-");
main(root, { test: null });
