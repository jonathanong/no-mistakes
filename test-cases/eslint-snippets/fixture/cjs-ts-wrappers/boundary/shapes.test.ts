const { vi: api = renamed, ...rest } = require("vitest");
api.mock("./dep");
void rest;
void renamed;

const { jest: jestFn } = require("@jest/globals");
jestFn.mock("./other");

const spec = "vitest";
const { [spec]: dynamic } = require(spec);
dynamic.mock("./dynamic");

const jestNs = require("@jest/globals");
const { jest: jestFromNs } = jestNs;
jestFromNs.mock("./from-namespace");

const viName = "vi";
const { [viName]: hiddenVi } = require("vitest");
hiddenVi.mock("./hidden");
