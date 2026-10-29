// no-mistakes-disable-file test-no-live-repository-scan -- explicit uncertain loaders
import { createRequire } from "node:module";
createRequire(import.meta.url)("node:fs");
