import fs, { readFileSync as read } from "node:fs";
import { writeFile as write } from "fs/promises";
import * as child from "node:child_process";
import { execSync, spawnSync as run } from "child_process";
import { resolve } from "node:path";
import { tmpdir } from "node:os";

const input = "/tmp/input.json";
const output = resolve(tmpdir(), "output.json");
fs.renameSync(input, output);
read(input);
write(output, "/tmp/payload.txt");
fs.promises.copyFile("/tmp/source.json", "/var/tmp/destination.json");
fs.writeFileSync("output.txt", "/tmp/data.txt");

const workdir = "/tmp/workdir";
child.spawn("tool", ["/tmp/argument.json"], { cwd: workdir, env: { TMUX: "/tmp/tmux.sock" } });
run("tool", { cwd: "/var/tmp/working" });
execSync("tool", { ["cwd"]: resolve(tmpdir(), "working") });
child.execFileSync("/tmp/tool", [input], { cwd: "/tmp/exec-file" });
child.spawnSync("tool", ["/var/tmp/sync-argument.json"]);
child.exec("tool", { cwd: "/tmp/exec-working" });
const argv = ["/tmp/aliased-argv.json"];
const options = { cwd: "/tmp/aliased-options" };
child.spawn("tool", argv, options);

parseArgv(["--output", "/tmp/argv.json"]);
expect(result.path).toBe("/tmp/result.json");
expect(result).toEqual({ path: "/var/tmp/expected.json" });
mockS3.putObject({ Key: "/tmp/payload.json" });
const joinedForAssertion = resolve(tmpdir(), "assertion.json");
expect(result.path).toBe(joinedForAssertion);

const indirect = "/tmp/indirect.json";
const secondAlias = indirect;
read(secondAlias);
let reassignable = "/tmp/reassignable.json";
read(reassignable);
child.spawn("tool", { cwd: "/tmp/overridden", ...options });
child.spawn("tool", { cwd: "/tmp/duplicate", cwd: "." });

function shadowed(fs, child, read, run, vi, jest, require) {
  fs.readFileSync("/tmp/fake-fs");
  child.spawn("tool", ["/tmp/fake-child"]);
  read("/var/tmp/fake-read");
  run("tool", { cwd: "/tmp/fake-run" });
  vi.mock("node:fs");
  jest.mock("node:fs/promises");
  const localVitest = require("vitest");
  localVitest.vi.mock("node:fs");
}

import * as vitest from "vitest";
vitest.mock("node:fs");

process.env.TMUX = "/tmp/standalone-tmux.sock";
fs.realpath.native("/tmp/native-callback", callback);
fs.realpathSync.native("/var/tmp/native-sync");
