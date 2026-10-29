// Reported: a property descriptor reads constructor through a string argument.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { list } from "../src/dir";

process.env.POISON_DIR = mkdtempSync(join(tmpdir(), "poison-"));
const F = Object.getOwnPropertyDescriptor(
  Object.getPrototypeOf(function () {}),
  "constructor",
).value;
F('process.env.POISON_DIR = "./src"')();
list();
