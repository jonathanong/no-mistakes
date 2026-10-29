import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export const path = require("node:path");
export default require;
