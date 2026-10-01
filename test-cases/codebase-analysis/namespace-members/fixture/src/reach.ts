import * as whole from "./whole";
import { Copied, Used } from "./lib";

// `Used` is only built through a member; `Copied` is copied into a value, so
// the graph cannot tell what is built through the copy.
export const used = new Used.UsedClass();
export const copied = Copied;

export const everything = whole;
export const load = () => import("./dynamic");
export const required = require("./required");
