import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export const path = require("node:path");
export const api = (0, require)("../src/api.mjs");
