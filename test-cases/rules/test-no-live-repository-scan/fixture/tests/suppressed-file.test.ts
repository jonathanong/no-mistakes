// no-mistakes-disable-file test-no-live-repository-scan
// Negative: a file-level suppression covers every scan in the file.
import { readdirSync } from "node:fs";

readdirSync("docs");
readdirSync("packages");
