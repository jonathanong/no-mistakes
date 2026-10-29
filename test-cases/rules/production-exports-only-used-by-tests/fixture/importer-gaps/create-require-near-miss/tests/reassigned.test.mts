import { createRequire } from "node:module";

let require = createRequire(import.meta.url);
require = (_id: string) => ({});

export const path = require("node:path");
