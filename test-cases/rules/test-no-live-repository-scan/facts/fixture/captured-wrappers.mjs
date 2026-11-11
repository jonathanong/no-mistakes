import fs from "node:fs/promises";
import * as syncFs from "node:fs";
const originalReaddir = fs.readdir;
fs.readdir = (...args) => originalReaddir(...args);
const originalSync = syncFs.readdirSync;
syncFs.readdirSync = function (...args) { return originalSync(...args); };
fs.readdir = (...args) => {
  args[0] = process.cwd();
  return originalReaddir(...args);
};
fs.readdir = (...args) => originalReaddir(process.cwd(), ...args);
fs.readdir = (...args) => originalReaddir(...args.slice(1));
fs.opendir = (...args) => originalReaddir(...args);
const fakeFs = { readdir: originalReaddir };
const fakeOriginal = fakeFs.readdir;
fakeFs.readdir = (...args) => fakeOriginal(...args);
function shadowed(fs) {
  const original = fs.readdir;
  fs.readdir = (...args) => original(...args);
}
const originalReadFile = fs.readFile;
fs.readdir = (...args) => originalReadFile(...args);
let mutable = fs.readdir;
fs.readdir = (...args) => mutable(...args);
fs.readdir = (...args) => (() => originalReaddir(...args))();
fs.readdir = (...args) => { observe(args); return originalReaddir(...args); };
import fakeImported from "./fake-fs.mjs";
const fakeImportedOriginal = fakeImported.readdir;
fakeImported.readdir = (...args) => fakeImportedOriginal(...args);
const readFile = fs.readFile;
fs.readFile = (...args) => readFile(...args);
fs.readdir = (...args) => {
  if (args[0] === "/owned") throw new Error("fixture failure");
  return originalReaddir(...args);
};
fs.readdir = (...args) => {
  if (args.length !== 1) throw new Error("unexpected arity");
  return originalReaddir(...args);
};
fs.readdir = (...args) => {
  args.push(process.cwd());
  return originalReaddir(...args);
};
fs.readdir = (...args) => {
  mutate(args);
  return originalReaddir(...args);
};
fs.readdir = (...args) => {
  args[0].path = process.cwd();
  return originalReaddir(...args);
};
