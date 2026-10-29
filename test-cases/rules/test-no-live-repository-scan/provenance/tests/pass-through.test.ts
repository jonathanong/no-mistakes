// A mock that forwards its arguments unchanged is not a scan root of its own.
import fs from "node:fs";
import { vi } from "vitest";

const original = fs.readdirSync;

// Accepted: exact spread forwarding.
vi.spyOn(fs, "readdirSync").mockImplementation((...args) => original(...args));
vi.spyOn(fs, "readdirSync").mockImplementationOnce(function (...args) {
  return original(...args);
});

// Reported: the wrapper changes the arguments.
vi.spyOn(fs, "readdirSync").mockImplementation((...args) => original("./src", ...args));
vi.spyOn(fs, "readdirSync").mockImplementation((...args) => original(...args.slice(1)));
vi.spyOn(fs, "readdirSync").mockImplementation((first, ...args) => original(...args));
vi.spyOn(fs, "readdirSync").mockImplementation((...args) => original(...args, "extra"));
// Reported: forwarding outside a mock implementation.
const forward = (...args: [string]) => original(...args);
forward("./src");

// Reported: these mocks are not of an enumerating method, so forwarding is not pass-through.
vi.fn().mockImplementation((...args: [string]) => original(...args));
vi.spyOn(fs, "existsSync").mockImplementation((...args: [string]) => original(...args));

// Reported: the wrapper changes the forwarded arguments before the real call.
vi.spyOn(fs, "readdirSync").mockImplementation((...args: [string]) => {
  args[0] = "./src";
  return original(...args);
});
vi.spyOn(fs, "readdirSync").mockImplementation((...args: [string]) => {
  args.splice(0, 1, "./src");
  return original(...args);
});
vi.spyOn(fs, "readdirSync").mockImplementation((...args: [string]) => {
  args.length = 0;
  args.push("./src");
  return original(...args);
});
vi.spyOn(fs, "readdirSync").mockImplementation((...args: [string]) => {
  const patch = () => {
    args[0] = "./src";
  };
  patch();
  return original(...args);
});
