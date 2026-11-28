import type sdk = require("vitest");
import local = Namespace.Value;

const spec = "vitest";
const loaded = require(spec);
sdk.mock("./dep");
local.mock("./dep");
loaded.vi.unmock.call(loaded.vi, "./dep");
loaded.vi.doUnmock.apply(loaded.vi, ["./dep"]);
