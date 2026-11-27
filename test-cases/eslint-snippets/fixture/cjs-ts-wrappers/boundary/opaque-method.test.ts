import { vi } from "vitest";
import * as vt from "vitest";

const key = "ns";
vi[key].mock("./dep");
vt[key].unmock("react");
vt[key].doUnmock("react");

const spec = "vitest";
const loaded = require(spec);
loaded.vi.mock("./dynamic");
loaded.vi.doMock("./dynamic");
