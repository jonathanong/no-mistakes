// An erased type import does not shadow the runtime CommonJS loader.
import type { require } from "./require-factory";
const fs = require("node:fs");
fs.readdirSync("src");
