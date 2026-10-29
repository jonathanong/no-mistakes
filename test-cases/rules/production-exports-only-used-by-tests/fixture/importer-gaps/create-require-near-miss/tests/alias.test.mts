import { createRequire } from "node:module";

const loader = createRequire(import.meta.url);
const require = loader;

export const path = require("node:path");
