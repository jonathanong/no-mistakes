import { createRequire as makeRequire } from "node:module";
import * as nodeModule from "module";

const load = makeRequire(import.meta.url);
const again = nodeModule.createRequire(import.meta.url);
const defaults = builtins.createRequire(import.meta.url);
load("../widget.cjs");
again.resolve("../item.cjs");
defaults("../defaults.cjs");

import builtins from "node:module";
