import * as nodeModule from "module";

const require = nodeModule.createRequire(import.meta.url);

export const os = require("node:os");
