import * as nodeModule from "node:module";

// These analysis-only cases deliberately stay outside direct static loader forms.
const property = "createRequire";
nodeModule[property](import.meta.url)("node:fs");
module[false]("node:fs");
module["\uD800"]("node:fs");

const { createRequire: { nestedFactory } } = require("node:module");
nestedFactory(import.meta.url)("node:fs");
const [arrayFactory] = require("node:module");
arrayFactory(import.meta.url)("node:fs");

let missingInitializer;
missingInitializer("node:fs");
