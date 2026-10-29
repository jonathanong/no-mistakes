// A base other than this module changes relative resolution.
import { createRequire } from "node:module";

const require = createRequire("/elsewhere/");

export const path = require("./local.cjs");
