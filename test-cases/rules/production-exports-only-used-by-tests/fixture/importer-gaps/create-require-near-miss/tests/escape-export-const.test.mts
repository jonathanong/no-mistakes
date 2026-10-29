import { createRequire } from "node:module";

export const require = createRequire(import.meta.url);

export const api = require("../src/api.mjs");
