import fs from "node:fs";
function logicalAssignment() {
  const original = fs.readdir;
  fs.readdir ||= (...args) => original(...args);
}
function lexicalTarget() {
  const original = fs.readdir;
  let target;
  target = (...args) => original(...args);
}
function memberCallee() {
  fs.readdir = (...args) => fs.readdir(...args);
}
function lateCapture() {
  fs.readdir = (...args) => original(...args);
  const original = fs.readdir;
}
function destructuredCapture() {
  const { missing: original } = fs.readdir;
  fs.readdir = (...args) => original(...args);
}
function localFunctionCapture() {
  const original = () => [];
  fs.readdir = (...args) => original(...args);
}
function nestedReceiver() {
  const original = fs.promises.readdir;
  fs.promises.readdir = (...args) => original(...args);
}
function rewrittenRest() {
  const original = fs.readdir;
  fs.readdir = (...args) => { args = []; return original(...args); };
}
function excessiveTypeWrappers() {
  const original = fs.readdir;
  fs.readdir = ((...args) => original(...args)) as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown as unknown;
}
