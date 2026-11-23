/* global vi: writable */
import fs from "node:fs";
import { spawn } from "node:child_process";

fs.watch("/tmp/watched");
fs.createReadStream("/var/tmp/stream");
fs.readFileSync(...argumentsFromCaller, "/tmp/unknown-position");
spawn(...argumentsFromCaller);
spawn("tool", ...argumentsFromCaller);
spawn("tool");
spawn("tool", { env: { TMUX: "/tmp/environment-only" } });
spawn("tool", { cwd: "/tmp/overridden", [optionName]: "." });
spawn("tool", {
  get cwd() {
    return "/tmp/runtime-getter";
  },
});

vi = fakeRunner;
vi.mock("node:fs");
fs.readFileSync("/tmp/not-a-framework-mock");
