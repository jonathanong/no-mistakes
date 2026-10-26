import * as v from "vitest";
import vitest from "vitest";

declare const name: string;
const { mock } = v;
v.vi.mock("./a");
v.vi["mock"]("./b");
v.vi[`mock`]("./c");
(v.vi?.mock)("./d");
const f = v.vi.mock;
f("./e");
mock("./f");
v.vi.mock.call(v.vi, "./g");
(0, v.vi.mock)("./h");
const { mock: install = v.vi.mock, ...rest } = v;
install("./i");
function getVi() {
  return v.vi;
}
getVi().mock("./j");
vitest.mock("./default-is-not-vi");
v.vi[name]("./dynamic");
