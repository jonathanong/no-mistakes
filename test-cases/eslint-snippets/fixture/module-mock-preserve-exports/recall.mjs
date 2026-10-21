import { vi } from "vitest";

vi.mock.call(undefined, "./local", () => ({ charge() {} }));
vi.mock.apply(undefined, ["./local", () => ({ charge() {} })]);
vi[`mock`].call(undefined, "./local", () => ({ charge() {} }));
vi.mock.call(vi, "./internal");
vi.mock.apply(vi, ["./internal"]);
const call = "bind";
vi.mock[call](vi, "./internal", () => ({}));
