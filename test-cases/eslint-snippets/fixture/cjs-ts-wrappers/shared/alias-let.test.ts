const specifier = "vitest";
const { test: check } = require(specifier);
let run = check;
run("let", () => {});
const member = require(specifier).test;
let again = member;
again("let member", () => {});
