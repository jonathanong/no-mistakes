import { createRequire } from "node:module";
import { readdirSync as scanner } from "node:fs";
let readdirSync: any = createRequire(import.meta.url);
readdirSync = scanner;
readdirSync("fs"); // existing-scanner-after-reassignment
