// Getters, owned lets, parameters, and call arguments seen by the path-origin pass.
import { readdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { local } from "./local.mjs";

function decl() {
  return process.env.A;
}
const expr = function () {
  return process.env["B"] || "x";
};
const arrow = () => process.env.C ?? "x";
const block = () => {
  return process.env.D;
};
const withArg = (a) => process.env.E;
function* gen() {
  return process.env.F;
}
const twice = () => {
  const x = 1;
  return process.env.G;
};
let owned;
owned = mkdtempSync(tmpdir());
let mixed = mkdtempSync(tmpdir());
mixed = "./live";
let compound = mkdtempSync(tmpdir());
compound += "x";
let unset;
export let exported = mkdtempSync(tmpdir());
let destructured;
[destructured] = [mkdtempSync(tmpdir())];
let [pattern] = [mkdtempSync(tmpdir())];
export function param(first, second = "d", ...rest) {
  first = "x";
  readdirSync(first);
  readdirSync(second);
  readdirSync(rest);
  return () => [readdirSync(second)];
}
export function destructure({ a }, [b], c) {
  readdirSync(a);
  readdirSync(b);
  readdirSync(join(c, "x"));
}
readdirSync(decl());
readdirSync(expr());
readdirSync(arrow());
readdirSync(block());
readdirSync(withArg(1));
readdirSync(gen());
readdirSync(twice());
readdirSync(owned);
readdirSync(mixed);
readdirSync(compound);
readdirSync(unset);
readdirSync(exported);
readdirSync(destructured);
readdirSync(pattern);
readdirSync(missing());
readdirSync(process.env.H ?? "x");
readdirSync("x" && process.env.I);
readdirSync(process.env[String(1)]);
readdirSync(process.env.J && "x");
readdirSync(local("x", mkdtempSync(tmpdir()), "./relative", import.meta.dirname, process.cwd(), ...[1], mkdtempSync(tmpdir())));
const argArrow = (a) => process.env.K;
function noReturn() {
  process.env.L;
}
const notFunction = "x";
readdirSync(argArrow());
readdirSync(noReturn());
readdirSync(notFunction());
