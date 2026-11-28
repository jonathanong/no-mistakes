import { vi } from "vitest";
import * as vt from "vitest";

vi[1]("./numeric");
vi[`mock${""}`]("./template");

const alias = cycle;
const cycle = alias;
cycle.mock("./cycle");

vt.jest.mock("./dep");
