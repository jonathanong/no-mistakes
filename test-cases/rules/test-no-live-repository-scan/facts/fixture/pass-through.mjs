import fs, { readdirSync as original } from "node:fs";
import { vi } from "vitest";

vi.spyOn(fs, "readdirSync").mockImplementation((...args) => original(...args));
vi.spyOn(fs, "readdirSync").mockImplementationOnce(function (...args) {
  return original(...args);
});
vi.spyOn(fs, "readdirSync").mockImplementation((...args) => original("a", ...args));
vi.spyOn(fs, "readdirSync").mockImplementation((first, ...args) => original(...args));
vi.spyOn(fs, "readdirSync").mockImplementation((...args) => {
  args = [];
  return original(...args);
});
const wrap = (...args) => original(...args);
vi.spyOn(fs, "readdirSync").mockReturnValue((...args) => original(...args));
original(...wrap);
vi.spyOn(fs, "readdirSync").mockImplementation((...args) => original(...args.slice()));
vi.spyOn(fs, "readdirSync").mockImplementation(() => original(...unknownGlobal));
vi.spyOn(fs, "readdirSync").mockImplementation((...args) => original(...args, "extra"));
vi.spyOn(fs, "readdirSync").mockImplementation(function (...[first]) {
  return original(...first);
});
// Not attributable to an enumerating method: a bare mock, another method, a computed name.
vi.fn().mockImplementation((...args) => original(...args));
vi.spyOn(fs, "list").mockImplementation((...args) => original(...args));
vi.spyOn(fs, name).mockImplementation((...args) => original(...args));
foo((...args) => original(...args));
const mock = vi.fn();
mock.mockImplementation((...args) => original(...args));
getSpy()(fs, "readdirSync").mockImplementation((...args) => original(...args));
vi.spyOn(fs).mockImplementation((...args) => original(...args));
vi.spyOn(fs, ...names).mockImplementation((...args) => original(...args));
spyOn(fs, "readdirSync").mockImplementation((...args) => original(...args));
