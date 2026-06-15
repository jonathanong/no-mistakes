import { used } from "./util"; // resolves locally
import { nope } from "./missing"; // unresolved relative import
import { gone } from "@app/missing"; // matches the @app/* alias but target is missing
import { readFile } from "node:fs"; // external (Node builtin)
import express from "express"; // external (bare npm package)

export function broken() {
  used("x", { x: 1 });
  return [nope, gone, readFile, express];
}
