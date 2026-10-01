import * as whole from "./namespace-whole-target";
import { Backstop } from "./namespace-backstop-target";

export const everything = whole;
export const load = () => import("./namespace-dynamic-target");
export const required = require("./namespace-require-target");
export const missing = () => new Backstop.Missing();
