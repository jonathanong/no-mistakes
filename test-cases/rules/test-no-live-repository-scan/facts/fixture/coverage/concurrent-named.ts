import { it, type test as typedTest } from "vitest";
import { callback } from "./callbacks.mjs";
const expression = function expressionBody() {};
it.concurrent("function expression", expression);
let changed = () => {};
changed = () => {};
it.concurrent("changed callback", changed);
const scalar = 1;
it.concurrent("scalar callback", scalar);
it.concurrent("imported callback", callback);
function register(parameter) { it.concurrent("parameter callback", parameter); }
typedTest.concurrent("type only", () => {});
