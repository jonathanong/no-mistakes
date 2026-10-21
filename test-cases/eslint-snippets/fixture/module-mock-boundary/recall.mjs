import vitest from "vitest";
import * as ns from "vitest";
import { vi } from "vitest";

vitest.vi.mock("./local", () => ({ charge() {} }));
ns["vi"].mock("./local", () => ({ charge() {} }));
vi[`mock`]("./local", () => ({ charge() {} }));
(vi?.mock)("./local", () => ({ charge() {} }));
(0, vi.mock)("./local", () => ({ charge() {} }));
const { vi: mocked } = ns;
mocked.mock("./local", () => ({ charge() {} }));
